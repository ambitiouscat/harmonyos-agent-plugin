use std::path::Path;

/// Platform capability abstraction trait.
///
/// All IO operations performed by the agent core go through this trait,
/// decoupling agent logic from the specific platform runtime
/// (HarmonyOS sync / Desktop async / WASM).
pub trait HostCapabilities: Send + Sync {
    /// Read the full contents of a file within the sandbox.
    fn read_file(&self, path: &Path) -> Result<String, String>;

    /// Write content to a file within the sandbox, creating parent directories.
    fn write_file(&self, path: &Path, content: &str) -> Result<(), String>;

    /// Send a synchronous HTTP POST request with JSON body, return response body.
    fn http_post(&self, url: &str, body: &str) -> Result<String, String>;

    /// Execute a shell command in a subprocess, return stdout on success
    /// or stderr on non-zero exit.
    fn execute_command(&self, cmd: &str) -> Result<String, String>;

    /// Spawn a fire-and-forget task on a background thread.
    /// Used for concurrent tool execution when tools are marked `concurrent_safe`.
    fn spawn_task(&self, f: Box<dyn FnOnce() + Send>);
}
