use serde_json::Value;
use std::sync::Mutex;

// ── Hook event types ──

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookEvent {
    /// Fired when the user submits a prompt (before the agent loop starts).
    UserPromptSubmit,
    /// Fired before a tool is executed. Return Some(message) to block execution.
    PreToolUse,
    /// Fired after a tool has been executed.
    PostToolUse,
    /// Fired when the agent loop is about to stop.
    Stop,
}

// ── Hook context ──

#[derive(Debug, Clone)]
pub struct HookContext {
    pub event: HookEvent,
    pub tool_name: Option<String>,
    pub tool_args: Option<Value>,
    pub tool_output: Option<String>,
    pub user_query: Option<String>,
}

impl HookContext {
    pub fn for_user_prompt(query: &str) -> Self {
        Self {
            event: HookEvent::UserPromptSubmit,
            tool_name: None,
            tool_args: None,
            tool_output: None,
            user_query: Some(query.to_string()),
        }
    }

    pub fn for_pre_tool(name: &str, args: &Value) -> Self {
        Self {
            event: HookEvent::PreToolUse,
            tool_name: Some(name.to_string()),
            tool_args: Some(args.clone()),
            tool_output: None,
            user_query: None,
        }
    }

    pub fn for_post_tool(name: &str, args: &Value, output: &str) -> Self {
        Self {
            event: HookEvent::PostToolUse,
            tool_name: Some(name.to_string()),
            tool_args: Some(args.clone()),
            tool_output: Some(output.to_string()),
            user_query: None,
        }
    }

    pub fn for_stop() -> Self {
        Self {
            event: HookEvent::Stop,
            tool_name: None,
            tool_args: None,
            tool_output: None,
            user_query: None,
        }
    }
}

/// Type for hook callbacks. Return Some(String) to inject a message or block execution.
pub type HookCallback = Box<dyn Fn(&HookContext) -> Option<String> + Send + Sync>;

// ── Hook registry ──

pub struct HookRegistry {
    hooks: Mutex<Vec<(HookEvent, String, HookCallback)>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Mutex::new(Vec::new()),
        }
    }

    /// Register a hook callback for a specific event.
    /// `label` is a human-readable name for debugging.
    pub fn register<F>(&self, event: HookEvent, label: &str, callback: F)
    where
        F: Fn(&HookContext) -> Option<String> + Send + Sync + 'static,
    {
        let mut hooks = self.hooks.lock().unwrap();
        hooks.push((event, label.to_string(), Box::new(callback)));
    }

    /// Trigger all hooks for the given context.
    ///
    /// Returns a list of injection messages from hooks.
    /// For PreToolUse, any non-empty result means the tool is blocked.
    pub fn trigger(&self, ctx: &HookContext) -> Vec<String> {
        let hooks = self.hooks.lock().unwrap();
        hooks
            .iter()
            .filter(|(event, _, _)| *event == ctx.event)
            .filter_map(|(_, _, cb)| cb(ctx))
            .collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Global hook registry ──

static GLOBAL_HOOKS: std::sync::LazyLock<HookRegistry> =
    std::sync::LazyLock::new(HookRegistry::new);

pub fn global_hooks() -> &'static HookRegistry {
    &GLOBAL_HOOKS
}

/// Register a built-in log hook that prints tool calls to stdout.
pub fn register_default_hooks() {
    let hooks = global_hooks();

    hooks.register(HookEvent::PostToolUse, "log_tool_call", |ctx| {
        if let (Some(name), Some(args)) = (&ctx.tool_name, &ctx.tool_args) {
            let preview = serde_json::to_string(args)
                .unwrap_or_default();
            let preview = if preview.len() > 200 {
                format!("{}...", &preview[..200])
            } else {
                preview
            };
            println!("[hook] tool={} args={}", name, preview);
        }
        None // never block
    });

    hooks.register(HookEvent::UserPromptSubmit, "log_prompt", |ctx| {
        if let Some(query) = &ctx.user_query {
            println!("[hook] user_prompt len={}", query.len());
        }
        None
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_trigger() {
        let registry = HookRegistry::new();
        registry.register(HookEvent::PreToolUse, "test_block", |ctx| {
            if ctx.tool_name.as_deref() == Some("dangerous_tool") {
                Some("Blocked by test hook".into())
            } else {
                None
            }
        });

        let ctx = HookContext::for_pre_tool("dangerous_tool", &serde_json::json!({}));
        let results = registry.trigger(&ctx);
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("Blocked"));
    }

    #[test]
    fn test_hook_does_not_fire_wrong_event() {
        let registry = HookRegistry::new();
        registry.register(HookEvent::PreToolUse, "test", |ctx| {
            Some(format!("hooked: {:?}", ctx.tool_name))
        });

        // Fire a different event
        let ctx = HookContext::for_post_tool("read", &serde_json::json!({}), "output");
        let results = registry.trigger(&ctx);
        assert!(results.is_empty());
    }

    #[test]
    fn test_multiple_hooks_same_event() {
        let registry = HookRegistry::new();
        registry.register(HookEvent::Stop, "h1", |_| Some("h1".into()));
        registry.register(HookEvent::Stop, "h2", |_| Some("h2".into()));

        let ctx = HookContext::for_stop();
        let results = registry.trigger(&ctx);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_default_hooks_registered() {
        register_default_hooks();
        let hooks = global_hooks();
        let ctx = HookContext::for_post_tool("read", &serde_json::json!({"file": "test.txt"}), "content");
        let results = hooks.trigger(&ctx);
        // log_tool_call returns None, so no inject
        assert!(results.is_empty());
    }
}
