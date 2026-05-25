use crate::types::message::{AgentRequest, AgentResponse};
use std::sync::RwLock;

/// Global agent configuration injected by the host (configure action).
static AGENT_CONFIG: RwLock<serde_json::Value> =
    RwLock::new(serde_json::Value::Null);

pub fn get_config() -> serde_json::Value {
    AGENT_CONFIG.read().unwrap().clone()
}

pub fn dispatch(action: &str, args_json: &str) -> String {
    let request: Result<AgentRequest, _> = serde_json::from_str(args_json);

    let response = match action {
        "ping" => AgentResponse {
            status: "ok".into(),
            message: Some("pong".into()),
            error: None,
        },
        "load_session" => AgentResponse {
            status: "ok".into(),
            message: Some(r#"{"session_id":"stub"}"#.into()),
            error: None,
        },
        "configure" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(cfg) => {
                    if let Ok(mut w) = AGENT_CONFIG.write() {
                        *w = cfg;
                    }
                    AgentResponse {
                        status: "ok".into(),
                        message: Some("Configuration stored".into()),
                        error: None,
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid config JSON: {}", e)),
                },
            }
        }
        "test_stream" => match request {
            Ok(AgentRequest::TestStream {
                chunks,
                interval_ms,
            }) => {
                crate::sandbox::validate::start_stream_sim(chunks, interval_ms);
                AgentResponse {
                    status: "streaming".into(),
                    message: Some("Stream simulation started".into()),
                    error: None,
                }
            }
            _ => AgentResponse {
                status: "error".into(),
                message: None,
                error: Some("test_stream requires chunks and interval_ms params".into()),
            },
        },
        _ => AgentResponse {
            status: "error".into(),
            message: None,
            error: Some(format!("Unknown action: {}", action)),
        },
    };

    serde_json::to_string(&response).unwrap_or_else(|e| {
        format!(
            r#"{{"status":"error","error":"JSON serialize failed: {}"}}"#,
            e
        )
    })
}
