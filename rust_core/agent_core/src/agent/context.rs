use crate::types::message::{ContentPart, Message};

/// Rough token estimate: ~4 chars per token for English/Chinese mixed text.
const CHARS_PER_TOKEN: usize = 4;

/// Fraction of the context window that triggers compaction.
const COMPACTION_THRESHOLD: f64 = 0.8;

/// Number of recent messages to always preserve in full.
const KEEP_RECENT: usize = 4;

/// Context window manager.
///
/// Tracks estimated token usage and triggers compaction (summarising or
/// trimming older messages) when the usage exceeds 80 % of the budget.
#[derive(Debug, Clone)]
pub struct ContextManager {
    token_budget: usize,
}

impl ContextManager {
    pub fn new(token_budget: usize) -> Self {
        Self { token_budget }
    }

    /// Estimate tokens for a single message.
    pub fn estimate_tokens(msg: &Message) -> usize {
        let mut chars = msg.role.len();
        for part in &msg.parts {
            chars += match part {
                ContentPart::Text { text } => text.len(),
                ContentPart::Reasoning { text, .. } => text.len(),
                ContentPart::ToolCall {
                    name, arguments, ..
                } => name.len() + arguments.len(),
                ContentPart::ToolResult { output, .. } => output.len(),
            };
        }
        chars / CHARS_PER_TOKEN
    }

    /// Total estimated tokens across all messages.
    pub fn total_tokens(messages: &[Message]) -> usize {
        messages.iter().map(Self::estimate_tokens).sum()
    }

    /// Returns `true` when compaction should be triggered.
    pub fn needs_compaction(&self, messages: &[Message]) -> bool {
        let total = Self::total_tokens(messages);
        total as f64 > self.token_budget as f64 * COMPACTION_THRESHOLD
    }

    /// Compact messages in-place.
    ///
    /// Strategy (Phase 1 — structural trim only, no LLM summarisation):
    /// 1. System prompt (role == "system") is preserved verbatim.
    /// 2. The last `KEEP_RECENT` messages are preserved verbatim.
    /// 3. Older tool outputs are trimmed to 500 chars + "[truncated]".
    /// 4. Other older messages are left as-is for now (summarisation in
    ///    a later phase).
    pub fn compact(&self, messages: &mut Vec<Message>) {
        if messages.is_empty() {
            return;
        }

        let cut_point = messages.len().saturating_sub(KEEP_RECENT);

        for (idx, msg) in messages.iter_mut().enumerate() {
            if msg.role == "system" {
                continue; // system prompt: never touch
            }
            if idx >= cut_point {
                continue; // recent: preserve
            }

            // Trim tool outputs in old messages.
            for part in &mut msg.parts {
                if let ContentPart::ToolResult { output, .. } = part {
                    if output.len() > 500 {
                        output.truncate(500);
                        output.push_str("…[truncated]");
                    }
                }
            }
        }
    }

    pub fn budget(&self) -> usize {
        self.token_budget
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_msg(role: &str, text: &str) -> Message {
        Message {
            id: None,
            role: role.into(),
            parts: vec![ContentPart::Text {
                text: text.into(),
            }],
        }
    }

    #[test]
    fn test_token_estimation() {
        let msg = mk_msg("user", "hello world"); // 4 + 11 chars → 15/4 ≈ 3 tokens
        assert_eq!(ContextManager::estimate_tokens(&msg), 3);
    }

    #[test]
    fn test_needs_compaction() {
        let cm = ContextManager::new(100);
        // 1000 chars → ~250 tokens → over 80 % of 100
        let long = "x".repeat(1000);
        let msgs = vec![mk_msg("user", &long)];
        assert!(cm.needs_compaction(&msgs));
    }

    #[test]
    fn test_no_compaction_below_threshold() {
        let cm = ContextManager::new(1000);
        let msgs = vec![mk_msg("user", "hi")];
        assert!(!cm.needs_compaction(&msgs));
    }

    #[test]
    fn test_system_prompt_preserved() {
        let cm = ContextManager::new(100);
        let mut msgs = vec![
            Message {
                id: None,
                role: "system".into(),
                parts: vec![ContentPart::Text {
                    text: "you are helpful".into(),
                }],
            },
            mk_msg("user", "hi"),
            mk_msg("assistant", "hello"),
            mk_msg("user", "ok"),
            mk_msg("assistant", "sure"),
            mk_msg("user", "more"),
            mk_msg("assistant", "stuff"),
        ];
        cm.compact(&mut msgs);
        // system prompt should still be intact.
        assert_eq!(msgs[0].role, "system");
    }

    #[test]
    fn test_tool_output_trimmed_in_old_messages() {
        let cm = ContextManager::new(1000);
        let long_output = "x".repeat(2000);
        let mut msgs = vec![
            // An old message with a large tool result.
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
            // Enough recent messages to push the tool msg into "old" zone.
            mk_msg("user", "a"),
            mk_msg("assistant", "b"),
            mk_msg("user", "c"),
            mk_msg("assistant", "d"),
        ];
        cm.compact(&mut msgs);
        // The old tool output should be truncated.
        if let ContentPart::ToolResult { output, .. } = &msgs[0].parts[0] {
            assert!(output.len() <= 520); // 500 + "[truncated]" overhead
            assert!(output.contains("truncated"));
        } else {
            panic!("expected ToolResult");
        }
    }
}
