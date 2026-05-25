use serde::{Deserialize, Serialize};

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

/// Minimal message model (expanded in Phase 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
