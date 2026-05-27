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
    use agent_core::agent::session;
    let dir = std::env::temp_dir().join("hmos_test_ffi_sess");
    let _ = std::fs::remove_dir_all(&dir);
    session::init_session_manager(dir.to_str().unwrap());

    let resp = json_router::dispatch("load_session", r#"{"session_id":"abc"}"#);
    // Without creating a session first, load should return an error
    assert!(resp.contains(r#""status":"error""#));

    // Create a session, then load it
    let create_resp = json_router::dispatch("create_session", r#"{"title":"test"}"#);
    assert!(create_resp.contains(r#""status":"ok""#));
    // Extract session_id from create response
    let v: serde_json::Value = serde_json::from_str(&create_resp).unwrap();
    let session_json: serde_json::Value =
        serde_json::from_str(v["message"].as_str().unwrap()).unwrap();
    let sid = session_json["meta"]["id"].as_str().unwrap();

    let load_args = format!(r#"{{"session_id":"{}"}}"#, sid);
    let resp2 = json_router::dispatch("load_session", &load_args);
    assert!(resp2.contains(r#""status":"ok""#));
    assert!(resp2.contains(sid));
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
