use crate::agent::abort::ABORT_FLAG;
use crate::agent::pipeline::ReconnectConfig;
use crate::types::message::ChatMessage;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

/// Call LLM API via ureq, parse SSE response, emit text deltas with staggered timing.
/// Retries with exponential backoff on connection failure using ReconnectConfig.
/// Deduplicates overlapping content across retries to avoid repeated text.
/// ABORT_FLAG is checked before each delta — Stop() may be delayed up to one SSE frame.
pub fn chat_completion_ureq(
    config: &serde_json::Value,
    messages: &[ChatMessage],
    on_chunk: impl Fn(&str),
) -> Result<(), String> {
    let reconnect = ReconnectConfig::default();
    chat_completion_ureq_with_retry(config, messages, on_chunk, &reconnect)
}

pub fn chat_completion_ureq_with_retry(
    config: &serde_json::Value,
    messages: &[ChatMessage],
    on_chunk: impl Fn(&str),
    reconnect: &ReconnectConfig,
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

    // Keep the last N chars already emitted, used to dedup on reconnect
    let mut last_sent: String = String::new();
    let dedup_window: usize = 80;

    for attempt in 0..=reconnect.max_retries {
        if ABORT_FLAG.load(Ordering::Relaxed) {
            return Err("aborted".into());
        }

        if attempt > 0 {
            let delay = reconnect.delay_for_attempt(attempt - 1);
            // Notify UI about reconnection
            let reconnect_json = serde_json::json!({"type":"reconnect","attempt":attempt});
            on_chunk(&reconnect_json.to_string());
            thread::sleep(delay);
        }

        let resp = match ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send_json(&body)
        {
            Ok(r) => r,
            Err(e) => {
                if attempt < reconnect.max_retries { continue; }
                return Err(format!("HTTP request failed after {} retries: {}", attempt, e));
            }
        };

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

        let mut resp_body = resp.into_body();
        let text = match resp_body.read_to_string() {
            Ok(t) => t,
            Err(e) => {
                if attempt < reconnect.max_retries { continue; }
                return Err(format!("read error after {} retries: {}", attempt, e));
            }
        };

        // Parse SSE stream
        let events = text.split("\n\n");
        let mut chunk_index: u64 = 0;
        let mut got_data = false;
        // Track new content for dedup on this retry round
        let mut new_sent: String = String::new();

        for event in events {
            if ABORT_FLAG.load(Ordering::Relaxed) {
                if !new_sent.is_empty() {
                    // Update last_sent before returning
                    if new_sent.len() > dedup_window {
                        last_sent = new_sent[new_sent.len() - dedup_window..].into();
                    } else {
                        last_sent = new_sent.clone();
                    }
                }
                return Err("aborted".into());
            }
            let trimmed = event.trim();
            if trimmed.is_empty() { continue; }
            for line in trimmed.split('\n') {
                if !line.starts_with("data: ") { continue; }
                let data = line[6..].trim();
                if data == "[DONE]" {
                    return Ok(());
                }
                if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(delta) = chunk["choices"][0]["delta"]["content"].as_str() {
                        if !delta.is_empty() {
                            got_data = true;
                            // Dedup: skip overlapping content from retry
                            let deduped = if attempt > 0 {
                                dedup_delta(&last_sent, &new_sent, delta)
                            } else {
                                delta.to_string()
                            };

                            if !deduped.is_empty() {
                                new_sent.push_str(&deduped);
                                let delta_json = serde_json::json!({"type":"text","text":deduped});
                                on_chunk(&delta_json.to_string());
                                thread::sleep(Duration::from_millis(20));
                                chunk_index += 1;
                            }
                        }
                    }
                    if chunk["choices"][0]["finish_reason"].as_str().is_some() {
                        return Ok(());
                    }
                }
            }
        }

        if got_data {
            // Successful round, update last_sent
            if new_sent.len() > dedup_window {
                last_sent = new_sent[new_sent.len() - dedup_window..].into();
            } else {
                last_sent = new_sent;
            }
            return Ok(());
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
            return Ok(());
        }

        // If we got here with no data, retry if possible
        if attempt < reconnect.max_retries { continue; }
    }

    Err(format!("exhausted {} retries", reconnect.max_retries))
}

/// Skip prefix of `delta` that overlaps with `last_sent`.
fn dedup_delta(last_sent: &str, new_sent: &str, delta: &str) -> String {
    let combined = format!("{}{}", last_sent, new_sent);
    let overlap = find_overlap(&combined, delta);
    delta[overlap..].to_string()
}

/// Find the longest prefix of `delta` that appears as a suffix of `haystack`.
fn find_overlap(haystack: &str, delta: &str) -> usize {
    let max_check = haystack.len().min(delta.len()).min(200);
    for n in (1..=max_check).rev() {
        if delta.len() >= n && haystack.ends_with(&delta[..n]) {
            return n;
        }
    }
    0
}
