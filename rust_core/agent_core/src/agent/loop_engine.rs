use crate::agent::abort::ABORT_FLAG;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::atomic::Ordering;

/// Max allowed iterations before hard-stop (safety net).
const MAX_ITERATIONS: u32 = 30;

/// Number of recent tool-call rounds to fingerprint for loop detection.
const SLIDING_WINDOW_SIZE: usize = 5;

/// Outcome of a ReAct loop run.
#[derive(Debug, Clone)]
pub enum LoopOutcome {
    Completed { iterations: u32 },
    StoppedByUser,
    LoopDetected { round_hash: String },
    MaxIterationsReached,
}

/// Fingerprint of a single tool-call round.
#[derive(Debug, Clone)]
struct RoundFingerprint {
    tool_name: String,
    arguments_hash: String,
}

impl RoundFingerprint {
    fn new(name: &str, args: &str) -> Self {
        let mut h = Sha256::new();
        h.update(args.as_bytes());
        let hash = format!("{:x}", h.finalize());
        Self {
            tool_name: name.to_string(),
            arguments_hash: hash,
        }
    }
}

/// Sliding-window loop detector.
///
/// Records fingerprints of the last N tool-call rounds.
/// If the same fingerprint repeats inside the window the loop is detected.
struct LoopDetector {
    window: VecDeque<RoundFingerprint>,
}

impl LoopDetector {
    fn new() -> Self {
        Self {
            window: VecDeque::with_capacity(SLIDING_WINDOW_SIZE),
        }
    }

    fn record(&mut self, name: &str, args: &str) -> bool {
        let fp = RoundFingerprint::new(name, args);

        // Check for duplicate within the window.
        for existing in &self.window {
            if existing.tool_name == fp.tool_name
                && existing.arguments_hash == fp.arguments_hash
            {
                return false; // loop detected
            }
        }

        if self.window.len() >= SLIDING_WINDOW_SIZE {
            self.window.pop_front();
        }
        self.window.push_back(fp);
        true
    }
}

/// The ReAct loop controller.
///
/// Phase 1 provides the skeleton; full LLM-tool integration arrives in later
/// phases when `SystemCallbacks.post_fn` is wired.
pub struct AgentLoop {
    detector: LoopDetector,
    iteration: u32,
}

impl AgentLoop {
    pub fn new() -> Self {
        Self {
            detector: LoopDetector::new(),
            iteration: 0,
        }
    }

    /// Returns true when the loop may continue.
    pub fn can_continue(&self) -> bool {
        !ABORT_FLAG.load(Ordering::Relaxed) && self.iteration < MAX_ITERATIONS
    }

    /// Register a tool call in the sliding-window detector.
    ///
    /// Returns `true` if the call is allowed (not a loop), `false` if a loop
    /// was detected.
    pub fn record_tool_call(&mut self, name: &str, args: &str) -> bool {
        self.iteration += 1;
        self.detector.record(name, args)
    }

    /// Finalise the loop and return an outcome.
    pub fn finish(&self, stopped: bool) -> LoopOutcome {
        if stopped {
            LoopOutcome::StoppedByUser
        } else if self.iteration >= MAX_ITERATIONS {
            LoopOutcome::MaxIterationsReached
        } else {
            LoopOutcome::Completed {
                iterations: self.iteration,
            }
        }
    }
}

impl Default for AgentLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detection_duplicate_args() {
        let mut agent = AgentLoop::new();
        // First call — ok.
        assert!(agent.record_tool_call("search", r#"{"q":"hello"}"#));
        // Second call with SAME args — should be detected.
        assert!(!agent.record_tool_call("search", r#"{"q":"hello"}"#));
    }

    #[test]
    fn test_different_args_allowed() {
        let mut agent = AgentLoop::new();
        assert!(agent.record_tool_call("search", r#"{"q":"a"}"#));
        assert!(agent.record_tool_call("search", r#"{"q":"b"}"#));
        assert!(agent.record_tool_call("search", r#"{"q":"c"}"#));
    }

    #[test]
    fn test_max_iterations_hard_limit() {
        let mut agent = AgentLoop::new();
        for i in 0..30 {
            assert!(agent.can_continue());
            agent.record_tool_call("t", &format!(r#"{{"i":{}}}"#, i));
        }
        assert!(!agent.can_continue());
        match agent.finish(false) {
            LoopOutcome::MaxIterationsReached => {}
            _ => panic!("expected MaxIterationsReached"),
        }
    }

    #[test]
    fn test_user_stop() {
        let agent = AgentLoop::new();
        match agent.finish(true) {
            LoopOutcome::StoppedByUser => {}
            _ => panic!("expected StoppedByUser"),
        }
    }
}
