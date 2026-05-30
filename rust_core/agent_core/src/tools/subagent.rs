use serde_json::Value;
use std::path::PathBuf;

/// Maximum sub-agent nesting depth to prevent infinite recursive spawning.
const MAX_DEPTH: usize = 3;

/// Spawn a sub-agent to handle a task in isolation.
///
/// Uses `std::thread::spawn` to isolate from the parent agent loop, then calls
/// the shared `run_subagent_inner` from `agent::agent_pool`. The `ThreadPoolSpawner`
/// also calls the same shared function — this is how we break the circular dependency.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_subagent(description: &str, workdir: &PathBuf, depth: usize) -> Result<String, String> {
    if depth >= MAX_DEPTH {
        return Err(format!(
            "Maximum sub-agent nesting depth ({}) exceeded.",
            MAX_DEPTH
        ));
    }

    let desc = description.to_string();
    let wd = workdir.clone();

    let handle = std::thread::spawn(move || {
        crate::agent::agent_pool::run_subagent_inner(&desc, &wd, depth)
    });

    match handle.join() {
        Ok(result) => result,
        Err(_) => Err("Sub-agent thread panicked".into()),
    }
}

/// WASM stub — subagent requires threads.
#[cfg(target_arch = "wasm32")]
pub fn spawn_subagent(_description: &str, _workdir: &PathBuf, _depth: usize) -> Result<String, String> {
    Err("Sub-agents are not available on wasm32 target".into())
}

// ── ToolRegistry handler for the parent agent ──

pub fn subagent_handler(args: Value, sandbox_root: &str) -> Result<String, String> {
    let description = args["description"]
        .as_str()
        .ok_or_else(|| "Missing 'description' parameter".to_string())?;
    let depth = args["depth"].as_u64().unwrap_or(0) as usize;

    let workdir = PathBuf::from(sandbox_root);

    spawn_subagent(description, &workdir, depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_depth_limit() {
        let result = spawn_subagent(
            "test task",
            &PathBuf::from("."),
            MAX_DEPTH,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("depth"));
    }

    #[test]
    fn test_subagent_depth_allowed() {
        let result = spawn_subagent(
            "echo hello",
            &PathBuf::from("."),
            0,
        );
        if let Err(e) = &result {
            assert!(!e.contains("depth"), "Should not be depth error: {}", e);
        }
    }

    #[test]
    fn test_subagent_handler_missing_description() {
        let result = subagent_handler(serde_json::json!({}), "/tmp");
        assert!(result.is_err());
    }
}
