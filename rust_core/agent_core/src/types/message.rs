use serde::{Deserialize, Serialize};

// ── Agent Request & Response (Phase 0, preserved) ──

/// Incoming request from the host (ArkTS via NAPI).
/// The action is passed separately as a C string; the JSON body is untagged.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentRequest {
    RunStep { messages: Vec<Message> },
    LoadSession { session_id: String },
    TestStream {
        #[serde(default = "default_chunks")]
        chunks: u32,
        #[serde(default = "default_interval_ms")]
        interval_ms: u64,
    },
}

fn default_chunks() -> u32 {
    20
}

fn default_interval_ms() -> u64 {
    50
}

/// Response returned to the host.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Polymorphic Message Model (Phase 1) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<String>,
    pub role: String,
    pub parts: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "reasoning")]
    Reasoning { text: String, collapsed: bool },

    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        name: String,
        output: String,
        is_error: bool,
    },
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_part_roundtrip() {
        let msg = Message {
            id: None,
            role: "assistant".into(),
            parts: vec![ContentPart::Text {
                text: "Hello".into(),
            }],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, "assistant");
        assert_eq!(parsed.parts.len(), 1);
    }

    #[test]
    fn test_content_part_tagged_serialization() {
        let cp = ContentPart::ToolCall {
            id: "tc1".into(),
            name: "search".into(),
            arguments: r#"{"query":"test"}"#.into(),
        };
        let json = serde_json::to_string(&cp).unwrap();
        assert!(json.contains(r#""type":"tool_call""#));
    }

    #[test]
    fn test_reasoning_part() {
        let cp = ContentPart::Reasoning {
            text: "thinking...".into(),
            collapsed: false,
        };
        let json = serde_json::to_string(&cp).unwrap();
        let back: ContentPart = serde_json::from_str(&json).unwrap();
        match back {
            ContentPart::Reasoning { text, collapsed } => {
                assert_eq!(text, "thinking...");
                assert!(!collapsed);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_tool_result_error_flag() {
        let cp = ContentPart::ToolResult {
            id: "tr1".into(),
            name: "search".into(),
            output: "file not found".into(),
            is_error: true,
        };
        let json = serde_json::to_string(&cp).unwrap();
        assert!(json.contains("\"is_error\":true"));
    }

    #[test]
    fn test_agent_request_ping_untagged() {
        let json = r#"{}"#;
        let _req: AgentRequest = serde_json::from_str(json).unwrap();
        // unknown {} matches TestStream with defaults
    }

    #[test]
    fn test_agent_request_run_step() {
        let json = r#"{"messages":[{"id":null,"role":"user","parts":[{"type":"text","text":"hi"}]}]}"#;
        let req: AgentRequest = serde_json::from_str(json).unwrap();
        match req {
            AgentRequest::RunStep { messages } => assert_eq!(messages.len(), 1),
            _ => panic!("expected RunStep"),
        }
    }
}
