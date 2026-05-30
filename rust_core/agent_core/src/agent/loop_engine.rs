use crate::agent::abort::ABORT_FLAG;
use crate::agent::platform::HostCapabilities;
use crate::agent::tool_registry::ToolRegistry;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::Ordering;

const MAX_ITERATIONS: u32 = 30;
const SLIDING_WINDOW_SIZE: usize = 5;

// ── Loop outcome ──

#[derive(Debug, Clone)]
pub enum LoopOutcome {
    Completed { iterations: u32 },
    StoppedByUser,
    LoopDetected { round_hash: String },
    MaxIterationsReached,
}

// ── LLM response types ──

#[derive(Debug, Clone)]
pub struct ToolCallBlock {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub reasoning_text: String,
    pub tool_calls: Vec<ToolCallBlock>,
    pub stop_reason: String,
}

// ── Loop detector ──

#[derive(Debug, Clone)]
struct RoundFingerprint {
    tool_name: String,
    arguments_hash: String,
}

impl RoundFingerprint {
    fn new(name: &str, args: &str) -> Self {
        let mut h = Sha256::new();
        h.update(args.as_bytes());
        Self {
            tool_name: name.to_string(),
            arguments_hash: format!("{:x}", h.finalize()),
        }
    }
}

struct LoopDetector {
    window: VecDeque<RoundFingerprint>,
    /// Track consecutive repeats of the same fingerprint for two-level self-healing.
    consecutive_repeats: u32,
    last_fingerprint: Option<RoundFingerprint>,
}

impl LoopDetector {
    fn new() -> Self {
        Self {
            window: VecDeque::with_capacity(SLIDING_WINDOW_SIZE),
            consecutive_repeats: 0,
            last_fingerprint: None,
        }
    }

    /// Record a tool call. Returns a LoopAction:
    /// - Continue: allowed
    /// - Warn(String): 1-3 repeats — inject corrective prompt
    /// - Abort(String): 5 repeats — hard abort
    fn record(&mut self, name: &str, args_json: &str) -> LoopAction {
        let fp = RoundFingerprint::new(name, args_json);

        // Check consecutive repeats (same as immediate last fingerprint)
        if let Some(ref last) = self.last_fingerprint {
            if last.tool_name == fp.tool_name && last.arguments_hash == fp.arguments_hash {
                self.consecutive_repeats += 1;
            } else {
                self.consecutive_repeats = 0;
            }
        }

        self.last_fingerprint = Some(fp.clone());

        // Level 2: hard abort at 5 consecutive repeats
        if self.consecutive_repeats >= 5 {
            return LoopAction::Abort(format!(
                "Loop detected: '{}' called 5 times with identical arguments. Aborting.",
                name
            ));
        }

        // Level 1: inject corrective prompt at 1-3 consecutive repeats
        if self.consecutive_repeats >= 1 && self.consecutive_repeats <= 3 {
            return LoopAction::Warn(format!(
                "You just called '{}' with the same arguments as the previous round. \
                 If this is intentional, explain why. Otherwise, try a different approach.",
                name
            ));
        }

        // Check sliding window for non-consecutive repeats
        for existing in &self.window {
            if existing.tool_name == fp.tool_name
                && existing.arguments_hash == fp.arguments_hash
            {
                return LoopAction::Warn(format!(
                    "Tool '{}' was called with identical arguments earlier. Re-evaluate your approach.",
                    name
                ));
            }
        }

        if self.window.len() >= SLIDING_WINDOW_SIZE {
            self.window.pop_front();
        }
        self.window.push_back(fp);

        LoopAction::Continue
    }
}

pub enum LoopAction {
    Continue,
    Warn(String),
    Abort(String),
}

// ── Agent Loop ──

/// Call the LLM API (streaming) with tools support.
/// Streams text deltas via `on_text` and returns accumulated tool_use blocks.
///
/// `host` is accepted for future migration of HTTP to `HostCapabilities`.
/// Currently ureq is used directly for SSE streaming since `HostCapabilities`
/// does not yet expose a streaming HTTP primitive.
#[cfg(not(target_arch = "wasm32"))]
fn llm_api_call(
    _host: &dyn HostCapabilities,
    config: &serde_json::Value,
    messages: &[serde_json::Value],
    tools: &[serde_json::Value],
    on_text: &(impl Fn(&str) + ?Sized),
) -> Result<LlmResponse, String> {
    let api_key = config["api_key"].as_str().ok_or("missing api_key")?;
    let base_url = config["base_url"].as_str().ok_or("missing base_url")?;
    let model = config["model"].as_str().ok_or("missing model")?;
    let max_tokens = config["max_tokens"].as_u64().unwrap_or(32000);

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "max_tokens": max_tokens,
    });

    if !tools.is_empty() {
        body["tools"] = serde_json::Value::Array(tools.to_vec());
    }

    // Use agent with http_status_as_error(false) to capture API error body on 4xx/5xx
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();

    let resp = agent.post(&url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if resp.status().as_u16() >= 400 {
        let status = resp.status().as_u16();
        let body_str = resp.into_body().read_to_string().unwrap_or_default();
        let err_detail = serde_json::from_str::<serde_json::Value>(&body_str)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(String::from))
            .unwrap_or_else(|| format!("HTTP {} — {}", status, &body_str[..body_str.len().min(300)]));
        return Err(err_detail);
    }

// ── SSE streaming parser ──
// Supports both Claude API (content_block_start/delta) and OpenAI API (delta.content/tool_calls).
//
// TODO: Add Claude-native API output format support (messages_to_api).
// Currently only OpenAI format is emitted. Claude's API expects a different
// tool_use/tool_result structure in the messages array. When adding Claude output,
// detect the provider from config and switch formats accordingly.
    let mut full_text = String::new();
    let mut reasoning_text = String::new();
    let mut tool_calls: Vec<ToolCallBlock> = Vec::new();
    let mut stop_reason = String::from("stop");
    struct ToolFrag { id: String, name: String, args: String }
    let mut tool_fragments: HashMap<usize, ToolFrag> = HashMap::new();
    let mut openai_fragments: HashMap<usize, (String, String, String)> = HashMap::new();

    let reader = resp.into_body().into_reader();
    use std::io::{BufRead, BufReader};
    let buf_reader = BufReader::new(reader);

    for line_result in buf_reader.lines() {
        // Abort check mid-stream
        if ABORT_FLAG.load(Ordering::Relaxed) {
            return Err("aborted".into());
        }

        let line = line_result.map_err(|e| format!("SSE read error: {}", e))?;
        let line = line.trim().to_string();
        if line.is_empty() { continue; }

        let data = if let Some(rest) = line.strip_prefix("data: ") {
            rest
        } else if let Some(rest) = line.strip_prefix("data:") {
            rest
        } else {
            continue;
        };

        if data == "[DONE]" { continue; }

        let chunk: serde_json::Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // ── Claude format: content_block_start ──
        if let Some(cb) = chunk["content_block_start"].as_object() {
            if cb["type"] == "tool_use" {
                let idx = cb["index"].as_u64().unwrap_or(0) as usize;
                let id = cb["id"].as_str().unwrap_or("").to_string();
                let name = cb["name"].as_str().unwrap_or("").to_string();
                tool_fragments.insert(idx, ToolFrag { id, name, args: String::new() });
            }
            continue;
        }

        // ── Claude format: content_block_delta ──
        if let Some(cbd) = chunk["content_block_delta"].as_object() {
            let idx = cbd["index"].as_u64().unwrap_or(0) as usize;
            // TEXT delta (THIS WAS MISSING — caused all text to vanish)
            if let Some(text) = cbd["text_delta"].as_str() {
                full_text.push_str(text);
                on_text(text);
            }
            // Tool input delta
            if let Some(input_json) = cbd["input_json_delta"].as_str() {
                if let Some(frag) = tool_fragments.get_mut(&idx) {
                    frag.args.push_str(input_json);
                }
            }
            continue;
        }

        // ── Claude format: stop_reason at top level ──
        if let Some(reason) = chunk["stop_reason"].as_str() {
            if !reason.is_empty() {
                stop_reason = reason.to_string();
            }
        }

        // ── OpenAI format: choices[0].delta ──
        if let Some(delta) = chunk["choices"].get(0).and_then(|c| c.get("delta")) {
            // Text
            if let Some(t) = delta["content"].as_str() {
                full_text.push_str(t);
                on_text(t);
            }
            // Reasoning/thinking content (DeepSeek R1, etc.) — must be echoed back
            if let Some(r) = delta["reasoning_content"].as_str() {
                reasoning_text.push_str(r);
            }
            // Tool calls
            if let Some(tc_array) = delta["tool_calls"].as_array() {
                for tc in tc_array {
                    let idx = tc["index"].as_u64().unwrap_or(0) as usize;
                    let (ref mut id, ref mut name, ref mut args) =
                        openai_fragments.entry(idx).or_insert_with(|| (String::new(), String::new(), String::new()));
                    if let Some(v) = tc["id"].as_str() { *id = v.to_string(); }
                    if let Some(v) = tc["function"]["name"].as_str() { *name = v.to_string(); }
                    if let Some(v) = tc["function"]["arguments"].as_str() { args.push_str(v); }
                }
            }
        }

        // ── OpenAI format: finish_reason ──
        if let Some(reason) = chunk["choices"][0]["finish_reason"].as_str() {
            if !reason.is_empty() {
                stop_reason = reason.to_string();
            }
        }
    }

    // Flush OpenAI tool fragments
    for (_, (id, name, args_str)) in openai_fragments.iter() {
        if !id.is_empty() && !name.is_empty() {
            let raw_args: serde_json::Value = serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));
            let args = normalize_args(&raw_args);
            tool_calls.push(ToolCallBlock { id: id.clone(), name: name.clone(), arguments: args });
        }
    }

    // Flush Claude tool fragments
    for (_, frag) in tool_fragments.iter() {
        if !frag.id.is_empty() && !frag.name.is_empty() {
            let raw_args: serde_json::Value = serde_json::from_str(&frag.args).unwrap_or(serde_json::json!({}));
            let args = normalize_args(&raw_args);
            if !tool_calls.iter().any(|tc| tc.id == frag.id) {
                tool_calls.push(ToolCallBlock { id: frag.id.clone(), name: frag.name.clone(), arguments: args });
            }
        }
    }

    Ok(LlmResponse {
        text: full_text,
        reasoning_text,
        tool_calls,
        stop_reason,
    })
}

/// WASM stub — agent loop is not available in browser context.
#[cfg(target_arch = "wasm32")]
fn llm_api_call(
    _host: &dyn HostCapabilities,
    _config: &serde_json::Value,
    _messages: &[serde_json::Value],
    _tools: &[serde_json::Value],
    _on_text: &(impl Fn(&str) + ?Sized),
) -> Result<LlmResponse, String> {
    Err("Agent loop LLM calls are not available on wasm32 target".into())
}

/// Normalize tool arguments to a JSON object (never null).
fn normalize_args(args: &serde_json::Value) -> serde_json::Value {
    if args.is_null() || !args.is_object() {
        serde_json::json!({})
    } else {
        args.clone()
    }
}

/// Convert internal Message (with ContentParts) to OpenAI-compatible API JSON.
fn messages_to_api(messages: &[crate::types::message::Message]) -> Vec<serde_json::Value> {
    let mut result: Vec<serde_json::Value> = Vec::new();
    use crate::types::message::ContentPart;

    for m in messages {
        // Check if pure tool results — emit as role:"tool" messages
        if !m.parts.is_empty()
            && m.parts.iter().all(|p| matches!(p, ContentPart::ToolResult { .. }))
        {
            for p in &m.parts {
                if let ContentPart::ToolResult { id, output, is_error, .. } = p {
                    result.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": id,
                        "content": if *is_error { format!("Error: {}", output) } else { output.clone() }
                    }));
                }
            }
            continue;
        }

        // Extract text and tool_calls from parts
        let mut text_parts: Vec<&str> = Vec::new();
        let mut reasoning_parts: Vec<&str> = Vec::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();

        for p in &m.parts {
            match p {
                ContentPart::Text { text } => text_parts.push(text),
                ContentPart::ToolCall { id, name, arguments } => {
                    let safe_args = normalize_args(arguments);
                    tool_calls.push(serde_json::json!({
                        "id": id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": serde_json::to_string(&safe_args).unwrap_or_default()
                        }
                    }));
                }
                ContentPart::ToolResult { id, output, is_error, .. } => {
                    result.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": id,
                        "content": if *is_error { format!("Error: {}", output) } else { output.clone() }
                    }));
                }
                ContentPart::Reasoning { text, .. } => {
                    reasoning_parts.push(text);
                }
            }
        }

        let text_content = text_parts.join("");
        let reasoning_content = reasoning_parts.join("");
        let has_text = !text_content.is_empty();
        let has_reasoning = !reasoning_content.is_empty();
        let has_tools = !tool_calls.is_empty();

        // Skip empty messages
        if !has_text && !has_reasoning && !has_tools {
            continue;
        }

        let mut obj = serde_json::json!({"role": m.role.clone()});
        if has_text {
            obj["content"] = serde_json::Value::String(text_content);
        }
        // Echo back reasoning_content for APIs that require it (DeepSeek R1, etc.)
        if has_reasoning && m.role == "assistant" {
            obj["reasoning_content"] = serde_json::Value::String(reasoning_content);
        }

        if has_tools {
            obj["tool_calls"] = serde_json::json!(tool_calls);
        }

        result.push(obj);
    }

    result
}

fn trigger_stop_hook() {
    let ctx = crate::agent::hooks::HookContext::for_stop();
    crate::agent::hooks::global_hooks().trigger(&ctx);
}

/// The V2 agent loop — `while stop_reason == "tool_use"`.
///
/// All IO operations (HTTP, files, commands) are routed through `host`.
/// Streaming LLM calls use the platform's HTTP capabilities for the POST request,
/// with SSE parsing remaining in-core for chunk delivery via C ABI callbacks.
///
/// Not available on wasm32 (no HTTP client).
#[cfg(not(target_arch = "wasm32"))]
pub fn agent_loop_run(
    host: &dyn HostCapabilities,
    config: &serde_json::Value,
    messages: &mut Vec<crate::types::message::Message>,
    registry: &ToolRegistry,
    on_text: impl Fn(&str),
    on_tool_call: impl Fn(&str, &str),
) -> Result<LoopOutcome, String> {
    let mut detector = LoopDetector::new();
    let mut iteration: u32 = 0;

    let tools = registry.get_schemas();

    // Trigger UserPromptSubmit hook
    {
        let query = messages
            .last()
            .and_then(|m| m.parts.first())
            .and_then(|p| match p {
                crate::types::message::ContentPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let ctx = crate::agent::hooks::HookContext::for_user_prompt(&query);
        crate::agent::hooks::global_hooks().trigger(&ctx);
    }

    loop {
        if ABORT_FLAG.load(Ordering::Relaxed) {
            trigger_stop_hook();
            return Ok(LoopOutcome::StoppedByUser);
        }
        if iteration >= MAX_ITERATIONS {
            trigger_stop_hook();
            return Ok(LoopOutcome::MaxIterationsReached);
        }

        // ── Context Compaction (7.7) ──
        let cm = crate::agent::context::ContextManager::with_default_budget();
        if cm.needs_compaction(messages) {
            crate::agent::compaction::compact_all(messages);
        }

        // Convert to API format
        let api_messages = messages_to_api(messages);

        // LLM call (streaming — text deltas go to on_text callback)
        let response = llm_api_call(host, config, &api_messages, &tools, &on_text)?;

        // Build assistant message parts
        let mut assistant_parts: Vec<crate::types::message::ContentPart> = Vec::new();

        if !response.reasoning_text.is_empty() {
            assistant_parts.push(crate::types::message::ContentPart::Reasoning {
                text: response.reasoning_text.clone(),
                collapsed: true,
            });
        }

        if !response.text.is_empty() {
            assistant_parts.push(crate::types::message::ContentPart::Text {
                text: response.text.clone(),
            });
        }

        for tc in &response.tool_calls {
            assistant_parts.push(crate::types::message::ContentPart::ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: tc.arguments.clone(),
            });
        }

        // Push assistant message
        messages.push(crate::types::message::Message {
            id: None,
            role: "assistant".into(),
            parts: assistant_parts,
        });

        // If no tool calls, we're done
        if response.tool_calls.is_empty() {
            trigger_stop_hook();
            return Ok(LoopOutcome::Completed {
                iterations: iteration,
            });
        }

        // Execute tools
        let mut tool_result_parts: Vec<crate::types::message::ContentPart> = Vec::new();

        for tc in &response.tool_calls {
            let args_str = serde_json::to_string(&tc.arguments).unwrap_or_default();

            // Loop detection
            match detector.record(&tc.name, &args_str) {
                LoopAction::Abort(reason) => {
                    trigger_stop_hook();
                    return Ok(LoopOutcome::LoopDetected {
                        round_hash: reason,
                    });
                }
                LoopAction::Warn(warning) => {
                    // Inject warning as a tool result so the LLM sees it
                    tool_result_parts.push(crate::types::message::ContentPart::ToolResult {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        output: format!("[System warning: {}]", warning),
                        is_error: false,
                    });
                    // Also push a system message with the warning
                    messages.push(crate::types::message::Message {
                        id: None,
                        role: "user".into(),
                        parts: vec![crate::types::message::ContentPart::Text {
                            text: format!(
                                "<system-reminder>{}</system-reminder>",
                                warning
                            ),
                        }],
                    });
                    continue; // Skip this tool call on warn
                }
                LoopAction::Continue => {}
            }

            on_tool_call(&tc.name, &args_str);

            // ── Hook: PreToolUse (7.3) ──
            {
                let ctx = crate::agent::hooks::HookContext::for_pre_tool(&tc.name, &tc.arguments);
                let blocks: Vec<String> = crate::agent::hooks::global_hooks().trigger(&ctx);
                if !blocks.is_empty() {
                    tool_result_parts.push(crate::types::message::ContentPart::ToolResult {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        output: format!("Tool blocked by hook: {}", blocks.join("; ")),
                        is_error: true,
                    });
                    continue;
                }
            }

            // ── Permission Pipeline (7.2) ──
            let sandbox_root = config["sandbox_root"].as_str().unwrap_or("");
            match crate::agent::permission::check_permission(&tc.name, &tc.arguments, sandbox_root) {
                crate::agent::permission::PermissionResult::Deny(reason) => {
                    tool_result_parts.push(crate::types::message::ContentPart::ToolResult {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        output: format!("Permission denied: {}", reason),
                        is_error: true,
                    });
                    continue;
                }
                crate::agent::permission::PermissionResult::AskUser { .. } => {
                    let approved = crate::agent::permission::wait_for_user_approval(
                        &tc.name,
                        &serde_json::to_string(&tc.arguments).unwrap_or_default(),
                    );
                    if !approved {
                        tool_result_parts.push(crate::types::message::ContentPart::ToolResult {
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                            output: "User denied permission for this operation.".into(),
                            is_error: true,
                        });
                        continue;
                    }
                }
                crate::agent::permission::PermissionResult::Allow => {}
            }

            // Execute via ToolRegistry
            let output = match registry.dispatch(&tc.name, tc.arguments.clone()) {
                Ok(out) => out,
                Err(e) => format!("Error: {}", e),
            };

            // ── Hook: PostToolUse (7.3) ──
            {
                let ctx = crate::agent::hooks::HookContext::for_post_tool(
                    &tc.name,
                    &tc.arguments,
                    &output,
                );
                crate::agent::hooks::global_hooks().trigger(&ctx);
            }

            tool_result_parts.push(crate::types::message::ContentPart::ToolResult {
                id: tc.id.clone(),
                name: tc.name.clone(),
                output,
                is_error: false,
            });
        }

        // Push tool results as user message
        if !tool_result_parts.is_empty() {
            messages.push(crate::types::message::Message {
                id: None,
                role: "user".into(),
                parts: tool_result_parts,
            });
        }

        iteration += 1;

        // ── Nag reminder + auto-complete (7.4) ──
        {
            let task_arc = crate::agent::task_state::global_task_store();
            let mut store = task_arc.lock().unwrap();
            if let Some(nag) = store.end_round_nag() {
                messages.push(crate::types::message::Message {
                    id: None,
                    role: "user".into(),
                    parts: vec![crate::types::message::ContentPart::Text { text: nag }],
                });
            }
            if store.all_completed() {
                trigger_stop_hook();
                return Ok(LoopOutcome::Completed {
                    iterations: iteration,
                });
            }
        }
    }
}

/// Convenience: run agent loop with simple text-only messages (backward compat).
#[cfg(not(target_arch = "wasm32"))]
pub fn agent_loop_run_simple(
    host: &dyn HostCapabilities,
    config: &serde_json::Value,
    user_message: &str,
    registry: &ToolRegistry,
    on_text: impl Fn(&str),
    on_tool_call: impl Fn(&str, &str),
) -> Result<LoopOutcome, String> {
    let mut messages = vec![crate::types::message::Message {
        id: None,
        role: "user".into(),
        parts: vec![crate::types::message::ContentPart::Text {
            text: user_message.to_string(),
        }],
    }];

    agent_loop_run(host, config, &mut messages, registry, on_text, on_tool_call)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detector_continue() {
        let mut d = LoopDetector::new();
        match d.record("search", r#"{"q":"a"}"#) {
            LoopAction::Continue => {}
            _ => panic!("expected Continue"),
        }
    }

    #[test]
    fn test_loop_detector_warn_on_repeat() {
        let mut d = LoopDetector::new();
        d.record("search", r#"{"q":"a"}"#);
        match d.record("search", r#"{"q":"a"}"#) {
            LoopAction::Warn(_) => {}
            _ => panic!("expected Warn on first repeat"),
        }
    }

    #[test]
    fn test_loop_detector_abort_after_5() {
        let mut d = LoopDetector::new();
        // First call: Continue (no history)
        d.record("search", r#"{"q":"a"}"#);
        // Calls 2-5: Warn (consecutive repeats 1-4, last 3 trigger Warn)
        for _ in 0..4 {
            let action = d.record("search", r#"{"q":"a"}"#);
            assert!(matches!(action, LoopAction::Warn(_)));
        }
        // Call 6: 5th consecutive repeat → Abort
        match d.record("search", r#"{"q":"a"}"#) {
            LoopAction::Abort(_) => {}
            other => panic!("expected Abort after 5 consecutive repeats, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn test_loop_detector_different_args_reset() {
        let mut d = LoopDetector::new();
        d.record("search", r#"{"q":"a"}"#);
        d.record("search", r#"{"q":"a"}"#); // warn
        match d.record("search", r#"{"q":"b"}"#) {
            LoopAction::Continue => {}
            _ => panic!("expected Continue after different args"),
        }
    }

    #[test]
    fn test_messages_to_api_simple_text() {
        let msgs = vec![crate::types::message::Message {
            id: None,
            role: "user".into(),
            parts: vec![crate::types::message::ContentPart::Text {
                text: "hello".into(),
            }],
        }];
        let api = messages_to_api(&msgs);
        assert_eq!(api.len(), 1);
        assert_eq!(api[0]["role"], "user");
        assert_eq!(api[0]["content"], "hello");
    }

    #[test]
    fn test_messages_to_api_multi_part() {
        use crate::types::message::{ContentPart, Message};
        let msgs = vec![Message {
            id: None,
            role: "user".into(),
            parts: vec![
                ContentPart::Text { text: "result:".into() },
                ContentPart::ToolResult {
                    id: "tc1".into(),
                    name: "read".into(),
                    output: "file content".into(),
                    is_error: false,
                },
            ],
        }];
        let api = messages_to_api(&msgs);
        // Tool results should be split into separate messages
        assert!(!api.is_empty());
        // First message: text part
        let text_msg = api.iter().find(|v| v["role"] == "user");
        assert!(text_msg.is_some());
        // Tool result as separate message
        let tool_msg = api.iter().find(|v| v["role"] == "tool");
        assert!(tool_msg.is_some());
        assert_eq!(tool_msg.unwrap()["tool_call_id"], "tc1");
        assert_eq!(tool_msg.unwrap()["content"], "file content");
    }
}
