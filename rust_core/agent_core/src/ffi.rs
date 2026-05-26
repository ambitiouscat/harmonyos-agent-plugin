use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::OnceLock;

// --- Type aliases for function pointers in SystemCallbacks ---
pub type PostFn = extern "C" fn(url: *const c_char, body: *const c_char) -> *mut c_char;
pub type StreamPostFn = extern "C" fn(
    url: *const c_char,
    body: *const c_char,
    on_chunk: extern "C" fn(chunk_data: *const c_char, event_type: u8),
) -> bool;
pub type FreeStrFn = extern "C" fn(ptr: *mut c_char);

/// Host platform IO capabilities injected at init time.
/// All fields are C-compatible raw function pointers.
/// `Option<extern "C" fn>` has guaranteed null-pointer optimization:
/// None is represented as NULL, identical to C behavior.
#[repr(C)]
pub struct SystemCallbacks {
    /// Blocking HTTP POST. Returns heap-allocated response JSON.
    pub post_fn: Option<PostFn>,
    /// Streaming HTTP POST. Host calls `on_chunk` for each received chunk.
    pub stream_post_fn: Option<StreamPostFn>,
    /// Free a string allocated by the host or Rust.
    pub free_str_fn: Option<FreeStrFn>,
}

pub static CALLBACKS: OnceLock<SystemCallbacks> = OnceLock::new();

/// Initialize the Rust core with host IO capabilities.
/// Must be called exactly once before any other FFI call.
#[no_mangle]
pub extern "C" fn rust_agent_init(callbacks: SystemCallbacks) -> bool {
    CALLBACKS.set(callbacks).is_ok()
}

/// Unified JSON message router.
///
/// # Safety
/// `action` and `json_args` must be valid null-terminated UTF-8 C strings.
/// The returned pointer must be freed by calling `rust_agent_free_str`.
#[no_mangle]
pub unsafe extern "C" fn rust_agent_call(
    action: *const c_char,
    json_args: *const c_char,
) -> *mut c_char {
    let action_str = CStr::from_ptr(action).to_str().unwrap_or("");
    let args_str = CStr::from_ptr(json_args).to_str().unwrap_or("{}");

    let response_json = crate::json_router::dispatch(action_str, args_str);

    CString::new(response_json)
        .unwrap_or_else(|_| {
            CString::new(r#"{"status":"error","error":"CString null byte"}"#).unwrap()
        })
        .into_raw()
}

/// Free a string previously returned by `rust_agent_call`.
///
/// # Safety
/// `ptr` must have been returned by `rust_agent_call` and not yet freed.
#[no_mangle]
pub unsafe extern "C" fn rust_agent_free_str(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Search a directory tree for lines matching `pattern` (in-process ripgrep).
/// Not available on wasm32 (no filesystem access).
///
/// # Safety
/// `dir_path` and `pattern` must be valid null-terminated UTF-8 C strings.
/// The returned pointer must be freed by calling `rust_agent_free_str`.
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe extern "C" fn rust_agent_search(
    dir_path: *const c_char,
    pattern: *const c_char,
) -> *mut c_char {
    let dir = CStr::from_ptr(dir_path).to_str().unwrap_or(".");
    let pat = CStr::from_ptr(pattern).to_str().unwrap_or("");

    let results = crate::agent::search::InProcessSearcher::search(Path::new(dir), pat);

    let json = match results {
        Ok(matches) => serde_json::to_string(&matches).unwrap_or_else(|_| "[]".into()),
        Err(e) => format!(r#"{{"status":"error","error":"{}"}}"#, e),
    };

    CString::new(json)
        .unwrap_or_else(|_| CString::new("[]").unwrap())
        .into_raw()
}

/// Scan a directory and build the BM25 RAG index in-memory.
/// Not available on wasm32 (no filesystem access).
///
/// # Safety
/// `dir_path` must be a valid null-terminated UTF-8 C string.
/// The returned pointer must be freed by calling `rust_agent_free_str`.
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe extern "C" fn rust_agent_scan_dir(dir_path: *const c_char) -> *mut c_char {
    let dir = CStr::from_ptr(dir_path).to_str().unwrap_or(".");
    let mut index = crate::agent::rag::RagIndex::new();

    let json = match index.scan_dir(Path::new(dir)) {
        Ok(count) => format!(r#"{{"status":"ok","chunks_indexed":{}}}"#, count),
        Err(e) => format!(r#"{{"status":"error","error":"{}"}}"#, e),
    };

    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"status":"error"}"#).unwrap())
        .into_raw()
}
