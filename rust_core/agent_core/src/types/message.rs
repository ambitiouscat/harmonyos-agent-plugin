use serde::{Deserialize, Serialize};

/// Incoming request from the host (ArkTS via NAPI).
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AgentRequest {
    #[serde(rename = "run_step")]
    RunStep { messages: Vec<Message> },
    #[serde(rename = "load_session")]
    LoadSession { session_id: String },
    #[serde(rename = "test_stream")]
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
