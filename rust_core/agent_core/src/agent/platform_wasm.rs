use crate::agent::platform::HostCapabilities;
use std::path::Path;

/// WASM platform host — all IO operations return "not supported" errors.
/// WASM lacks filesystem, threads, subprocesses, and direct HTTP in the browser context.
pub struct WasmHost;

impl WasmHost {
    pub fn new() -> Self {
        Self
    }
}

impl HostCapabilities for WasmHost {
    fn read_file(&self, _path: &Path) -> Result<String, String> {
        Err("File read is not supported in WASM environment".into())
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), String> {
        Err("File write is not supported in WASM environment".into())
    }

    fn http_post(&self, _url: &str, _body: &str) -> Result<String, String> {
        Err("HTTP requests are not supported in WASM environment".into())
    }

    fn execute_command(&self, _cmd: &str) -> Result<String, String> {
        Err("Shell execution is not supported in WASM environment".into())
    }

    fn spawn_task(&self, _f: Box<dyn FnOnce() + Send>) {
        // No-op: WASM has no threads
    }
}
