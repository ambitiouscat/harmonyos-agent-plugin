use agent_core::json_router;
use agent_core::types::message::{AgentRequest, AgentResponse};

#[test]
fn test_ping_dispatch() {
    let resp = json_router::dispatch("ping", "{}");
    assert!(resp.contains("pong"));
    assert!(resp.contains(r#""status":"ok""#));
}

#[test]
fn test_unknown_action() {
    let resp = json_router::dispatch("bogus_action", "{}");
    assert!(resp.contains("Unknown action"));
    assert!(resp.contains(r#""status":"error""#));
}

#[test]
fn test_load_session_dispatch() {
    let resp = json_router::dispatch("load_session", r#"{"action":"load_session","session_id":"abc"}"#);
    assert!(resp.contains("stub"));
    assert!(resp.contains(r#""status":"ok""#));
}

#[test]
fn test_agent_request_deser_ping_via_untagged() {
    // "ping" doesn't serialize as a tagged variant — it's matched by action string in the router.
    // This test validates that our message types round-trip correctly.
    let req = AgentRequest::LoadSession {
        session_id: "test123".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: AgentRequest = serde_json::from_str(&json).unwrap();
    match parsed {
        AgentRequest::LoadSession { session_id } => assert_eq!(session_id, "test123"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_agent_response_serialization() {
    let resp = AgentResponse {
        status: "ok".into(),
        message: Some("hello".into()),
        error: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("hello"));
    // error field should not appear when None
    assert!(!json.contains("error"));
}

#[test]
fn test_test_file_sandbox() {
    let dir = std::env::temp_dir();
    let dir_cstr = std::ffi::CString::new(dir.to_str().unwrap()).unwrap();
    let ok = unsafe { agent_core::sandbox::validate::test_file(dir_cstr.as_ptr()) };
    assert!(ok, "test_file should succeed in a writable temp dir");
}

#[test]
fn test_test_file_null_ptr() {
    let ok = unsafe { agent_core::sandbox::validate::test_file(std::ptr::null()) };
    assert!(!ok, "test_file should return false for NULL pointer");
}
