use crate::agent::abort::ABORT_FLAG;
use crate::types::message::ChatMessage;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

/// Call LLM API via ureq, parse SSE response, emit text deltas with staggered timing.
/// Uses ureq's built-in timeout (30s connect, 120s read) to bound the request.
/// ABORT_FLAG is checked before each delta — Stop() may be delayed up to one SSE frame.
/// Returns `Ok(())` on clean completion, `Err(msg)` on error or abort.
pub fn chat_completion_ureq(
    config: &serde_json::Value,
    messages: &[ChatMessage],
    on_chunk: impl Fn(&str),
) -> Result<(), String> {
    let api_key = config["api_key"].as_str().ok_or("missing api_key")?;
    let base_url = config["base_url"].as_str().ok_or("missing base_url")?;
    let model = config["model"].as_str().ok_or("missing model")?;
    let max_tokens = config["max_tokens"].as_u64().unwrap_or(32000);

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "max_tokens": max_tokens,
    });

    let resp = ureq::post(&url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if resp.status().as_u16() >= 400 {
        let status = resp.status().as_u16();
        let mut body_obj = resp.into_body();
        let body_str = body_obj.read_to_string().unwrap_or_default();
        let err_msg = serde_json::from_str::<serde_json::Value>(&body_str)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(String::from))
            .unwrap_or_else(|| format!("HTTP {} {}", status, body_str));
        return Err(err_msg);
    }

    let mut body = resp.into_body();
    let text = body
        .read_to_string()
        .map_err(|e| format!("read error: {}", e))?;

    // Parse SSE stream, emit deltas with inter-chunk delay
    let events = text.split("\n\n");
    let mut chunk_index: u64 = 0;

    for event in events {
        if ABORT_FLAG.load(Ordering::Relaxed) {
            return Err("aborted".into());
        }
        let trimmed = event.trim();
        if trimmed.is_empty() {
            continue;
        }
        for line in trimmed.split('\n') {
            if !line.starts_with("data: ") {
                continue;
            }
            let data = line[6..].trim();
            if data == "[DONE]" {
                return Ok(());
            }
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(delta) = chunk["choices"][0]["delta"]["content"].as_str() {
                    if !delta.is_empty() {
                        let delta_json = serde_json::json!({"type":"text","text":delta});
                        on_chunk(&delta_json.to_string());
                        thread::sleep(Duration::from_millis(20));
                        chunk_index += 1;
                    }
                }
                if chunk["choices"][0]["finish_reason"].as_str().is_some() {
                    return Ok(());
                }
            }
        }
    }

    if chunk_index == 0 {
        // Fallback: try non-streaming response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                if !content.is_empty() {
                    let delta_json = serde_json::json!({"type":"text","text":content});
                    on_chunk(&delta_json.to_string());
                }
            }
            if json["error"]["message"].as_str().is_some() {
                return Err(json["error"]["message"].as_str().unwrap().into());
            }
        }
    }

    Ok(())
}
