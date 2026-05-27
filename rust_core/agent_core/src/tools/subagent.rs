use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentConfig {
    pub name: String,
    pub system_prompt: String,
    pub max_iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentProgress {
    pub agent_name: String,
    pub iteration: u32,
    pub status: String, // "running", "completed", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Lightweight sub-agent runner that spawns an agent loop in a dedicated thread.
///
/// Each sub-agent runs synchronously in its own OS thread with its own
/// configuration scoped away from the main agent loop.
pub struct SubAgentRunner;

impl SubAgentRunner {
    /// Run a sub-agent in a background thread.
    /// Returns immediately with a JoinHandle; use `join()` or poll for results.
    pub fn run_async<F>(
        config: SubAgentConfig,
        on_progress: F,
    ) -> thread::JoinHandle<SubAgentProgress>
    where
        F: Fn(&SubAgentProgress) + Send + 'static,
    {
        thread::spawn(move || {
            let mut progress = SubAgentProgress {
                agent_name: config.name.clone(),
                iteration: 0,
                status: "running".into(),
                output: None,
                error: None,
            };

            for i in 0..config.max_iterations {
                // Check abort flag
                if crate::agent::abort::ABORT_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
                    progress.status = "cancelled".into();
                    progress.iteration = i;
                    on_progress(&progress);
                    return progress;
                }

                progress.iteration = i + 1;
                on_progress(&progress);

                // In a full implementation, this would:
                // 1. Call the LLM with the sub-agent's system prompt + task
                // 2. Parse tool calls from the response
                // 3. Execute tools and feed back results
                // 4. Check for completion signal
                //
                // For now, this is a structural stub ready for integration.
                thread::sleep(std::time::Duration::from_millis(10));
            }

            progress.status = "completed".into();
            progress.output = Some(format!(
                "Sub-agent '{}' completed {} iterations",
                config.name,
                config.max_iterations
            ));
            on_progress(&progress);
            progress
        })
    }

    /// Run a sub-agent synchronously (blocks the calling thread).
    pub fn run(config: SubAgentConfig) -> SubAgentProgress {
        let progress = Arc::new(Mutex::new(SubAgentProgress {
            agent_name: config.name.clone(),
            iteration: 0,
            status: "running".into(),
            output: None,
            error: None,
        }));

        let progress_clone = progress.clone();
        let handle = Self::run_async(config, move |p| {
            if let Ok(mut guard) = progress_clone.lock() {
                *guard = p.clone();
            }
        });

        match handle.join() {
            Ok(p) => p,
            Err(_) => SubAgentProgress {
                agent_name: "unknown".into(),
                iteration: 0,
                status: "error".into(),
                output: None,
                error: Some("Sub-agent thread panicked".into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_runner_sync() {
        let config = SubAgentConfig {
            name: "test-agent".into(),
            system_prompt: "You are a test agent.".into(),
            max_iterations: 3,
        };
        let result = SubAgentRunner::run(config);
        assert_eq!(result.status, "completed");
        assert_eq!(result.iteration, 3);
        assert!(result.output.is_some());
    }

    #[test]
    fn test_subagent_runner_async() {
        let config = SubAgentConfig {
            name: "async-test".into(),
            system_prompt: "Test".into(),
            max_iterations: 2,
        };
        let results = Arc::new(Mutex::new(vec![]));
        let r = results.clone();
        let handle = SubAgentRunner::run_async(config, move |p| {
            r.lock().unwrap().push(p.status.clone());
        });
        let final_progress = handle.join().unwrap();
        assert_eq!(final_progress.status, "completed");
        let statuses = results.lock().unwrap();
        assert!(!statuses.is_empty());
    }
}
