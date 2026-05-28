use crate::types::message::{ContentPart, Message};

const CHARS_PER_TOKEN: usize = 4;
const COMPACTION_THRESHOLD: f64 = 0.8;
const CONTEXT_BUDGET: usize = 180_000;

#[derive(Debug, Clone)]
pub struct ContextManager {
    token_budget: usize,
}

impl ContextManager {
    pub fn new(token_budget: usize) -> Self {
        Self { token_budget }
    }

    pub fn with_default_budget() -> Self {
        Self {
            token_budget: CONTEXT_BUDGET,
        }
    }

    pub fn estimate_tokens(msg: &Message) -> usize {
        let mut chars = msg.role.len();
        for part in &msg.parts {
            chars += match part {
                ContentPart::Text { text } => text.len(),
                ContentPart::Reasoning { text, .. } => text.len(),
                ContentPart::ToolCall { name, arguments, .. } => name.len() + serde_json::to_string(arguments).unwrap_or_default().len(),
                ContentPart::ToolResult { output, .. } => output.len(),
            };
        }
        chars / CHARS_PER_TOKEN
    }

    pub fn total_tokens(messages: &[Message]) -> usize {
        messages.iter().map(Self::estimate_tokens).sum()
    }

    pub fn needs_compaction(&self, messages: &[Message]) -> bool {
        let total = Self::total_tokens(messages);
        total as f64 > self.token_budget as f64 * COMPACTION_THRESHOLD
    }

    pub fn budget(&self) -> usize {
        self.token_budget
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
    fn test_token_estimation() {
        let msg = mk_msg("user", "hello world");
        assert_eq!(ContextManager::estimate_tokens(&msg), 3);
    }

    #[test]
    fn test_needs_compaction() {
        let cm = ContextManager::new(100);
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
}
