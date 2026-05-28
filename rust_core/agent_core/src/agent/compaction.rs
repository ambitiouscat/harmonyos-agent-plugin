use crate::types::message::{ContentPart, Message};

const TOOL_RESULT_BUDGET: usize = 200_000;
const MAX_MESSAGES: usize = 50;
const KEEP_RECENT_TOOL_RESULTS: usize = 3;

/// Run all compaction layers in order on the given messages.
pub fn compact_all(messages: &mut Vec<Message>) {
    // L1: tool_result budget — apply to last message
    let last_idx = messages.len().wrapping_sub(1);
    if let Some(last) = messages.get_mut(last_idx) {
        for part in &mut last.parts {
            if let ContentPart::ToolResult { output, .. } = part {
                if output.len() > TOOL_RESULT_BUDGET {
                    let kept = TOOL_RESULT_BUDGET / 2;
                    let half = kept / 2;
                    let tail_start = output.len().saturating_sub(half);
                    let tail = output[tail_start..].to_string();
                    *output = format!(
                        "{}\n\n[... large output ({} bytes) persisted ...]\n\n{}",
                        &output[..kept.min(output.len())],
                        output.len(),
                        tail,
                    );
                }
            }
        }
    }

    // L2: snip_compact
    if messages.len() > MAX_MESSAGES {
        let keep_head = 3usize;
        let keep_tail = 47usize;
        let snipped = messages.len().saturating_sub(keep_head + keep_tail);
        if snipped > 0 {
            let marker = Message {
                id: None,
                role: "system".into(),
                parts: vec![ContentPart::Text {
                    text: format!("[{} earlier messages snipped]", snipped),
                }],
            };
            let tail: Vec<Message> = messages.drain(keep_head..).collect();
            messages.truncate(keep_head);
            messages.push(marker);
            let skip_n = tail.len().saturating_sub(keep_tail);
            messages.extend(tail.into_iter().skip(skip_n));
        }
    }

    // L3: micro_compact
    let mut total_tr = 0usize;
    for msg in messages.iter() {
        for part in msg.parts.iter() {
            if matches!(part, ContentPart::ToolResult { .. }) {
                total_tr += 1;
            }
        }
    }
    let keep_start = total_tr.saturating_sub(KEEP_RECENT_TOOL_RESULTS);
    let mut tr_seen = 0usize;
    for msg in messages.iter_mut() {
        for part in msg.parts.iter_mut() {
            if let ContentPart::ToolResult { output, .. } = part {
                if tr_seen < keep_start && output.len() > 50 {
                    *output = "[Earlier tool result compacted.]".into();
                }
                tr_seen += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_msg(role: &str, text: &str) -> Message {
        Message {
            id: None,
            role: role.into(),
            parts: vec![ContentPart::Text { text: text.into() }],
        }
    }

    #[test]
    fn test_tool_output_compacted_in_old_messages() {
        let long_output = "x".repeat(2000);
        let mut msgs = vec![
            Message {
                id: None,
                role: "tool".into(),
                parts: vec![ContentPart::ToolResult {
                    id: "t1".into(),
                    name: "search".into(),
                    output: long_output,
                    is_error: false,
                }],
            },
            mk_msg("user", "a"),
            mk_msg("assistant", "b"),
            mk_msg("user", "c"),
            mk_msg("assistant", "d"),
        ];
        compact_all(&mut msgs);
        if let ContentPart::ToolResult { output, .. } = &msgs[0].parts[0] {
            assert!(!output.is_empty());
        } else {
            panic!("expected ToolResult");
        }
    }

    #[test]
    fn test_snip_compact_large_message_list() {
        let mut msgs: Vec<Message> = (0..60)
            .map(|i| mk_msg("user", &format!("msg {}", i)))
            .collect();
        let original_len = msgs.len();
        compact_all(&mut msgs);
        assert!(msgs.len() < original_len);
        let has_marker = msgs.iter().any(|m| {
            m.parts.iter().any(|p| matches!(p, ContentPart::Text { text } if text.contains("snipped")))
        });
        assert!(has_marker);
    }

    #[test]
    fn test_micro_compact_preserves_recent() {
        let mut msgs = vec![
            Message {
                id: None, role: "user".into(),
                parts: vec![ContentPart::ToolResult {
                    id: "old1".into(), name: "read".into(),
                    output: "old output 1".into(), is_error: false,
                }],
            },
            Message {
                id: None, role: "user".into(),
                parts: vec![ContentPart::ToolResult {
                    id: "old2".into(), name: "read".into(),
                    output: "old output 2".into(), is_error: false,
                }],
            },
            Message {
                id: None, role: "user".into(),
                parts: vec![ContentPart::ToolResult {
                    id: "recent".into(), name: "read".into(),
                    output: "recent output".into(), is_error: false,
                }],
            },
        ];
        compact_all(&mut msgs);
        if let ContentPart::ToolResult { output, .. } = &msgs[2].parts[0] {
            assert_eq!(output, "recent output");
        }
    }
}
