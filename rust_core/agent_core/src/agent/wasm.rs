#[cfg(feature = "wasm")]
use std::sync::OnceLock;
#[cfg(feature = "wasm")]
use std::ffi::{CStr, CString};
#[cfg(feature = "wasm")]
use std::os::raw::c_char;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Static storage for JS callbacks passed from the browser host.
/// OnceLock prevents GC from destroying the JS Function references.
#[cfg(feature = "wasm")]
static WASM_STREAM_CALLBACK: OnceLock<js_sys::Function> = OnceLock::new();
#[cfg(feature = "wasm")]
static WASM_IO_CALLBACK: OnceLock<js_sys::Function> = OnceLock::new();

/// C ABI bridge: invoked by Rust core when a stream chunk arrives.
/// Forwards the chunk data and event type back to the JS callback.
#[cfg(feature = "wasm")]
extern "C" fn wasm_on_chunk_bridge(chunk_data: *const c_char, event_type: u8) {
    if let Some(js_fn) = WASM_STREAM_CALLBACK.get() {
        let chunk_str = unsafe { CStr::from_ptr(chunk_data) }.to_str().unwrap_or("");
        let js_val = JsValue::from_str(chunk_str);
        let event_val = JsValue::from_f64(event_type as f64);
        let _ = js_fn.call2(&JsValue::NULL, &js_val, &event_val);
    }
}

/// Safe wrapper around rust_agent_free_str for wasm callbacks.
#[cfg(feature = "wasm")]
extern "C" fn wasm_free_str(ptr: *mut c_char) {
    unsafe { crate::ffi::rust_agent_free_str(ptr) }
}

/// C ABI bridge: invoked by Rust core for HTTP POST (IO proxy).
/// Calls into the JS IO callback and returns the response string.
#[cfg(feature = "wasm")]
extern "C" fn wasm_post_fn_bridge(url: *const c_char, body: *const c_char) -> *mut c_char {
    if let Some(js_fn) = WASM_IO_CALLBACK.get() {
        let url_str = unsafe { CStr::from_ptr(url) }.to_str().unwrap_or("");
        let body_str = unsafe { CStr::from_ptr(body) }.to_str().unwrap_or("");
        if let Ok(res_val) =
            js_fn.call2(&JsValue::NULL, &JsValue::from_str(url_str), &JsValue::from_str(body_str))
        {
            if let Some(res_str) = res_val.as_string() {
                return CString::new(res_str).unwrap().into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

/// Initialize the WASM agent with JS callbacks for streaming and IO.
/// Bridges JS Functions to C ABI function pointers via OnceLock.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_agent_init(on_chunk: js_sys::Function, on_io: js_sys::Function) -> bool {
    let _ = WASM_STREAM_CALLBACK.set(on_chunk);
    let _ = WASM_IO_CALLBACK.set(on_io);

    let cbs = crate::ffi::SystemCallbacks {
        post_fn: Some(wasm_post_fn_bridge),
        stream_post_fn: None,
        free_str_fn: Some(wasm_free_str),
    };

    crate::ffi::rust_agent_init(cbs)
}

/// Thin wrapper around the unified C ABI rust_agent_call for WASM.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_agent_call(action: &str, json_args: &str) -> String {
    let act = CString::new(action).unwrap();
    let args = CString::new(json_args).unwrap();
    let ptr = unsafe { crate::ffi::rust_agent_call(act.as_ptr(), args.as_ptr()) };
    let result = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("{}").to_string();
    unsafe { crate::ffi::rust_agent_free_str(ptr) };
    result
}
