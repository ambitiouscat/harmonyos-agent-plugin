use serde_json::Value;
use std::path::PathBuf;

/// Maximum sub-agent nesting depth to prevent infinite recursive spawning.
const MAX_DEPTH: usize = 3;

/// Spawn a sub-agent to handle a task in isolation.
///
/// The sub-agent gets fresh messages[], limited tools (no SubAgent to prevent
/// recursion), and a 30-turn safety limit. Only the final text summary is
/// returned — intermediate tool chatter is discarded.
///
/// # Depth Defense
/// Each level of sub-agent spawning increments `depth`. When `depth >= 3`, the
/// call is rejected to prevent infinite recursive spawning and memory exhaustion.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_subagent(description: &str, workdir: &PathBuf, depth: usize) -> Result<String, String> {
    if depth >= MAX_DEPTH {
        return Err(format!(
            "Maximum sub-agent nesting depth ({}) exceeded.",
            MAX_DEPTH
        ));
    }

    let desc = description.to_string();
    let wd = workdir.clone();

    // Run in a dedicated thread to isolate from the parent agent loop.
    let handle = std::thread::spawn(move || {
        run_subagent_inner(&desc, &wd, depth)
    });

    match handle.join() {
        Ok(result) => result,
        Err(_) => Err("Sub-agent thread panicked".into()),
    }
}

/// WASM stub — subagent requires threads.
#[cfg(target_arch = "wasm32")]
pub fn spawn_subagent(_description: &str, _workdir: &PathBuf, _depth: usize) -> Result<String, String> {
    Err("Sub-agents are not available on wasm32 target".into())
}

#[cfg(not(target_arch = "wasm32"))]
fn run_subagent_inner(description: &str, workdir: &PathBuf, _depth: usize) -> Result<String, String> {
    use crate::agent::loop_engine::agent_loop_run;
    use crate::agent::tool_registry::ToolRegistry;
    use crate::agent::platform_harmonyos::HarmonyOSHost;

    // Build a fresh ToolRegistry with only safe tools (no SubAgent).
    let mut registry = ToolRegistry::new(workdir.to_str().unwrap_or("."));
    register_sub_tools(&mut registry);

    // Build fresh messages with the sub-agent task.
    let mut messages = vec![crate::types::message::Message {
        id: None,
        role: "user".into(),
        parts: vec![crate::types::message::ContentPart::Text {
            text: format!(
                "{}\n\nComplete this task and return a concise summary. Do not delegate further.",
                description
            ),
        }],
    }];

    let config = crate::json_router::get_config();
    let sandbox_root = config["sandbox_root"].as_str().unwrap_or(".");
    let api_key = config["api_key"].as_str().unwrap_or("");
    let api_base_url = config["base_url"].as_str().unwrap_or("https://api.anthropic.com");
    let host = HarmonyOSHost::new(sandbox_root, api_key, api_base_url);

    // Suppress streaming output from sub-agent (only parent sees text).
    let on_text = |_text: &str| {};
    let on_tool = |_name: &str, _args: &str| {};

    // Run the agent loop with a fresh 30-turn limit.
    let outcome = agent_loop_run(&host, &config, &mut messages, &registry, on_text, on_tool)?;

    // Collect the final assistant text as a summary.
    let summary = messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant")
        .and_then(|m| {
            m.parts.iter().find_map(|p| match p {
                crate::types::message::ContentPart::Text { text } => Some(text.clone()),
                _ => None,
            })
        })
        .unwrap_or_else(|| format!("Sub-agent completed: {:?}", outcome));

    Ok(summary)
}

/// Register tools available to sub-agents (no SubAgent to prevent recursion).
#[cfg(not(target_arch = "wasm32"))]
fn register_sub_tools(registry: &mut crate::agent::tool_registry::ToolRegistry) {
    use crate::agent::tool_registry::ToolDef;

    registry.register(ToolDef {
        name: "read".into(),
        description: "Read a file from the filesystem.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string"}
            },
            "required": ["file_path"]
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::file::read_handler,
    });

    registry.register(ToolDef {
        name: "write".into(),
        description: "Write content to a file.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string"},
                "content": {"type": "string"}
            },
            "required": ["file_path", "content"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::file::write_handler,
    });

    registry.register(ToolDef {
        name: "edit".into(),
        description: "Edit a file by replacing an exact string match.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string"},
                "old_string": {"type": "string"},
                "new_string": {"type": "string"}
            },
            "required": ["file_path", "old_string", "new_string"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::edit::edit_handler,
    });

    // Note: NO SubAgent tool registered here — this is how we prevent infinite recursion.
}

// ── ToolRegistry handler for the parent agent ──

pub fn subagent_handler(args: Value, sandbox_root: &str) -> Result<String, String> {
    let description = args["description"]
        .as_str()
        .ok_or_else(|| "Missing 'description' parameter".to_string())?;
    let depth = args["depth"].as_u64().unwrap_or(0) as usize;

    let workdir = PathBuf::from(sandbox_root);

    spawn_subagent(description, &workdir, depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_depth_limit() {
        let result = spawn_subagent(
            "test task",
            &PathBuf::from("."),
            MAX_DEPTH, // depth already at limit
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("depth"));
    }

    #[test]
    fn test_subagent_depth_allowed() {
        // depth 0 should be allowed (simulates first call from main agent)
        let result = spawn_subagent(
            "echo hello",
            &PathBuf::from("."),
            0,
        );
        // On non-wasm32 this will try to run, on wasm32 it returns Err
        // But it shouldn't be a depth error
        if let Err(e) = &result {
            assert!(!e.contains("depth"), "Should not be depth error: {}", e);
        }
    }

    #[test]
    fn test_subagent_handler_missing_description() {
        let result = subagent_handler(serde_json::json!({}), "/tmp");
        assert!(result.is_err());
    }
}
