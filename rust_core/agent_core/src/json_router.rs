use crate::agent::abort::ABORT_FLAG;
use crate::sandbox::validate::STREAM_CB;
use crate::types::message::{AgentRequest, AgentResponse};
use std::ffi::CString;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::thread;

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
        "abort" => {
            ABORT_FLAG.store(true, Ordering::Relaxed);
            AgentResponse {
                status: "ok".into(),
                message: Some("Generation aborted".into()),
                error: None,
            }
        }
        "reset_abort" => {
            ABORT_FLAG.store(false, Ordering::Relaxed);
            AgentResponse {
                status: "ok".into(),
                message: Some("Abort flag reset".into()),
                error: None,
            }
        }
        "chat" => match request {
            Ok(AgentRequest::ChatStream { messages }) => {
                let config = AGENT_CONFIG.read().unwrap().clone();
                let stream_cb = {
                    let guard = STREAM_CB.lock().unwrap();
                    *guard
                };

                thread::spawn(move || {
                    let on_chunk = |chunk_json: &str| {
                        if let Some(cb) = stream_cb {
                            if let Ok(s) = CString::new(chunk_json) {
                                cb(s.as_ptr(), 0);
                            }
                        }
                    };
                    let result = crate::agent::chat::chat_completion_ureq(
                        &config,
                        &messages,
                        on_chunk,
                    );
                    if let Some(cb) = stream_cb {
                        match result {
                            Ok(()) => {
                                let done = CString::new("").unwrap();
                                cb(done.as_ptr(), 1);
                            }
                            Err(e) => {
                                let err_json = format!(
                                    r#"{{"type":"text","text":"{}","is_error":"true"}}"#,
                                    e.replace('"', r#"\""#)
                                );
                                if let Ok(s) = CString::new(err_json) {
                                    cb(s.as_ptr(), 2);
                                }
                            }
                        }
                    }
                });

                AgentResponse {
                    status: "streaming".into(),
                    message: Some("Chat streaming started".into()),
                    error: None,
                }
            }
            _ => AgentResponse {
                status: "error".into(),
                message: None,
                error: Some("chat requires messages array".into()),
            },
        },
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
