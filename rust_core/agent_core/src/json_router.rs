use crate::types::message::{AgentRequest, AgentResponse};

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
        "run_step" => AgentResponse {
            status: "ok".into(),
            message: Some("run_step stub: not implemented in Phase 0".into()),
            error: None,
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
