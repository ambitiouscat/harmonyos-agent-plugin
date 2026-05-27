#[cfg(feature = "node")]
use std::ffi::{CStr, CString};
#[cfg(feature = "node")]
use napi_derive::napi;

/// Safe wrapper around rust_agent_free_str to match non-unsafe FreeStrFn signature.
#[cfg(feature = "node")]
extern "C" fn node_free_str(ptr: *mut std::os::raw::c_char) {
    unsafe { crate::ffi::rust_agent_free_str(ptr) }
}

/// Initialize the Node.js agent with configuration JSON.
/// Node environment has native std::fs + ureq, so no JS IO callbacks needed.
#[cfg(feature = "node")]
#[napi]
pub fn node_agent_init(_config_json: Option<String>) -> bool {
    // Build SystemCallbacks with no-op IO — Node uses native Rust I/O
    let cbs = crate::ffi::SystemCallbacks {
        post_fn: None,
        stream_post_fn: None,
        free_str_fn: Some(node_free_str),
    };

    crate::ffi::rust_agent_init(cbs)
}

/// Thin wrapper around the unified C ABI rust_agent_call for Node.js.
#[cfg(feature = "node")]
#[napi]
pub fn node_agent_call(action: String, args: String) -> String {
    let act = CString::new(action).unwrap();
    let arg = CString::new(args).unwrap();
    let ptr = unsafe { crate::ffi::rust_agent_call(act.as_ptr(), arg.as_ptr()) };
    let result = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("{}").to_string();
    unsafe { crate::ffi::rust_agent_free_str(ptr) };
    result
}
