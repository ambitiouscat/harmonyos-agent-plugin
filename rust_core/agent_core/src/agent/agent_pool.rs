use crate::agent::platform::HostCapabilities;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Maximum sub-agent nesting depth.
pub const MAX_DEPTH: usize = 3;

// ── AgentHandle ──

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct AgentHandle {
    pub id: String,
    status: Arc<std::sync::Mutex<AgentStatus>>,
    cancelled: Arc<AtomicBool>,
    result_rx: mpsc::Receiver<Result<String, String>>,
}

impl AgentHandle {
    pub fn status(&self) -> AgentStatus {
        self.status.lock().unwrap().clone()
    }

    /// Block until the sub-agent completes, then return the result.
    pub fn result(self) -> Option<Result<String, String>> {
        match self.result_rx.recv() {
            Ok(result) => {
                let mut s = self.status.lock().unwrap();
                match &result {
                    Ok(_) => *s = AgentStatus::Completed,
                    Err(_) => *s = AgentStatus::Failed,
                }
                Some(result)
            }
            Err(_) => {
                let mut s = self.status.lock().unwrap();
                *s = AgentStatus::Cancelled;
                None
            }
        }
    }

    /// Request cancellation of the sub-agent.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        let mut s = self.status.lock().unwrap();
        *s = AgentStatus::Cancelled;
    }
}

// ── AgentConfig ──

pub struct AgentConfig<'a> {
    pub description: String,
    pub working_dir: PathBuf,
    pub depth: usize,
    pub host: &'a (dyn HostCapabilities + 'a),
}

// ── AgentSpawner trait ──

/// Abstraction for spawning sub-agents.
/// HarmonyOS: ThreadPoolSpawner (std::thread).
/// Desktop: can use TokioSpawner when feature = "async".
/// WASM: WasmAgentSpawner (always returns error).
pub trait AgentSpawner: Send + Sync {
    fn spawn(&self, config: AgentConfig) -> Result<AgentHandle, String>;
}

// ── ThreadPoolSpawner (HarmonyOS / Desktop) ──

#[cfg(not(target_arch = "wasm32"))]
pub struct ThreadPoolSpawner {
    max_threads: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl ThreadPoolSpawner {
    pub fn new(max_threads: usize) -> Self {
        Self { max_threads }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AgentSpawner for ThreadPoolSpawner {
    fn spawn(&self, config: AgentConfig) -> Result<AgentHandle, String> {
        if config.depth >= MAX_DEPTH {
            return Err(format!(
                "Maximum sub-agent nesting depth ({}) exceeded.",
                MAX_DEPTH
            ));
        }

        use std::sync::atomic::AtomicU64;
        static AGENT_COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = format!("agent-{}", AGENT_COUNTER.fetch_add(1, Ordering::Relaxed));
        let status = Arc::new(std::sync::Mutex::new(AgentStatus::Running));
        let cancelled = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();

        let status_clone = status.clone();
        let cancelled_clone = cancelled.clone();
        let desc = config.description.clone();
        let wd = config.working_dir.clone();
        let depth = config.depth;

        std::thread::spawn(move || {
            if cancelled_clone.load(Ordering::Relaxed) {
                let _ = tx.send(Err("Sub-agent cancelled before start.".into()));
                return;
            }

            // Sub-agent execution: spawn a fresh agent loop with limited tools.
            // In V1, this delegates to the existing subagent_handler via the
            // subagent tool, maintaining backward compatibility.
            let result = crate::tools::subagent::spawn_subagent(&desc, &wd, depth);
            let _ = tx.send(result);
        });

        Ok(AgentHandle {
            id,
            status,
            cancelled,
            result_rx: rx,
        })
    }
}

// ── WasmAgentSpawner (WASM stub) ──

#[cfg(target_arch = "wasm32")]
pub struct WasmAgentSpawner;

#[cfg(target_arch = "wasm32")]
impl WasmAgentSpawner {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
impl AgentSpawner for WasmAgentSpawner {
    fn spawn(&self, _config: AgentConfig) -> Result<AgentHandle, String> {
        Err("SubAgent spawning is not supported in WASM environment.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopHost;
    impl HostCapabilities for NoopHost {
        fn read_file(&self, _: &Path) -> Result<String, String> { Err("noop".into()) }
        fn write_file(&self, _: &Path, _: &str) -> Result<(), String> { Err("noop".into()) }
        fn http_post(&self, _: &str, _: &str) -> Result<String, String> { Err("noop".into()) }
        fn execute_command(&self, _: &str) -> Result<String, String> { Err("noop".into()) }
        fn spawn_task(&self, _: Box<dyn FnOnce() + Send>) {}
    }

    #[test]
    fn test_depth_limit_enforced() {
        let spawner = ThreadPoolSpawner::new(4);
        let cfg = AgentConfig {
            description: "test".into(),
            working_dir: PathBuf::from("."),
            depth: MAX_DEPTH, // exactly at limit → rejected
            host: &NoopHost,
        };
        assert!(spawner.spawn(cfg).is_err());
    }

    #[test]
    fn test_depth_allowed() {
        let spawner = ThreadPoolSpawner::new(4);
        let cfg = AgentConfig {
            description: "test".into(),
            working_dir: PathBuf::from("."),
            depth: 0,
            host: &NoopHost,
        };
        let handle = spawner.spawn(cfg).unwrap();
        assert!(!handle.id.is_empty());
        let status = handle.status();
        assert!(matches!(status, AgentStatus::Running));
    }
}
