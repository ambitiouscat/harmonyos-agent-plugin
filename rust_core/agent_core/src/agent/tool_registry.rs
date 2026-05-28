use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// Maximum tool output size in bytes before truncation kicks in.
const MAX_OUTPUT_BYTES: usize = 32_000;

/// Descriptor for a single tool registered in the system.
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub read_only: bool,
    pub concurrent_safe: bool,
    pub handler: fn(args: Value, sandbox_root: &str) -> Result<String, String>,
}

/// Central tool registry — singleton that holds all registered tools.
pub struct ToolRegistry {
    tools: HashMap<String, ToolDef>,
    sandbox_root: String,
}

impl ToolRegistry {
    pub fn new(sandbox_root: &str) -> Self {
        Self {
            tools: HashMap::new(),
            sandbox_root: sandbox_root.to_string(),
        }
    }

    /// Register a tool. Replaces any existing tool with the same name.
    pub fn register(&mut self, def: ToolDef) {
        self.tools.insert(def.name.clone(), def);
    }

    /// Return JSON Schema array for all registered tools (API format).
    pub fn get_schemas(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    }
                })
            })
            .collect()
    }

    /// Return tool names for introspection.
    pub fn get_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Check if a tool is registered.
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Dispatch a tool call by name. Returns the tool output (possibly truncated).
    pub fn dispatch(&self, name: &str, args: Value) -> Result<String, String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("Unknown tool: {}", name))?;

        let raw = (tool.handler)(args, &self.sandbox_root)?;
        Ok(truncate_output(raw))
    }
}

/// Smart truncation: keep first 50% + notice + last 25% when over limit.
/// Uses char boundaries for safe UTF-8 slicing.
fn truncate_output(output: String) -> String {
    if output.len() <= MAX_OUTPUT_BYTES {
        return output;
    }

    let half = MAX_OUTPUT_BYTES / 2;
    let quarter = MAX_OUTPUT_BYTES / 4;
    let total = output.len();

    // Find safe byte positions on char boundaries
    let head_end = output
        .char_indices()
        .take_while(|(i, _)| *i < half)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(half.min(total));

    let tail_start = output
        .char_indices()
        .rev()
        .take_while(|(i, _)| total - *i <= quarter)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(total.saturating_sub(quarter));

    let head = &output[..head_end];
    let tail = &output[tail_start..];
    let skipped = total - head_end - (total - tail_start);

    format!(
        "{}\n\n[... {} characters truncated ...]\n\n{}",
        head, skipped, tail
    )
}

/// Global singleton for the tool registry.
static GLOBAL_REGISTRY: RwLock<Option<ToolRegistry>> = RwLock::new(None);

/// Initialise the global tool registry. Must be called once before dispatch.
pub fn init_global_registry(sandbox_root: &str) {
    let mut registry = ToolRegistry::new(sandbox_root);
    register_builtin_tools(&mut registry);
    let mut guard = GLOBAL_REGISTRY.write().unwrap();
    *guard = Some(registry);
}

/// Register all Tier 1 built-in tools into the given registry.
fn register_builtin_tools(registry: &mut ToolRegistry) {
    // Read
    registry.register(ToolDef {
        name: "read".into(),
        description: "Read a file from the filesystem.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file to read"}
            },
            "required": ["file_path"]
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::file::read_handler,
    });

    // Write
    registry.register(ToolDef {
        name: "write".into(),
        description: "Write content to a file, creating it if necessary.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file to write"},
                "content": {"type": "string", "description": "Content to write to the file"}
            },
            "required": ["file_path", "content"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::file::write_handler,
    });

    // Edit
    registry.register(ToolDef {
        name: "edit".into(),
        description: "Edit a file by replacing an exact string match.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file to edit"},
                "old_string": {"type": "string", "description": "Exact string to find and replace"},
                "new_string": {"type": "string", "description": "Replacement string"}
            },
            "required": ["file_path", "old_string", "new_string"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::edit::edit_handler,
    });

    // Bash
    registry.register(ToolDef {
        name: "bash".into(),
        description: "Execute a bash command in a sandboxed environment.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "The bash command to execute"}
            },
            "required": ["command"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::bash::bash_handler,
    });

    // Glob
    registry.register(ToolDef {
        name: "glob".into(),
        description: "Find files matching a glob pattern (e.g. **/*.rs, src/**/*.ts).".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Glob pattern to match files against"}
            },
            "required": ["pattern"]
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::file::glob_handler,
    });

    // Grep
    registry.register(ToolDef {
        name: "grep".into(),
        description: "Search file contents using regex patterns.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Regex pattern to search for"},
                "path": {"type": "string", "description": "File or directory path (default: workspace root)"}
            },
            "required": ["pattern"]
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::file::grep_handler,
    });

    // Skill tool (Tier 2)
    registry.register(ToolDef {
        name: "skill_load".into(),
        description: "Load the full content of a skill by name for context injection.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Skill name (e.g. /search, /goal)"}
            },
            "required": ["name"]
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::skill::skill_load_handler,
    });

    // Memory tools (Tier 2)
    registry.register(ToolDef {
        name: "memory_save".into(),
        description: "Save a persistent memory for future sessions. Types: user, feedback, project, reference.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Short kebab-case slug (e.g. user-prefers-tabs)"},
                "description": {"type": "string", "description": "One-line description for the MEMORY.md index"},
                "memory_type": {"type": "string", "enum": ["user", "feedback", "project", "reference"]},
                "body": {"type": "string", "description": "Full memory content in markdown"}
            },
            "required": ["name", "description", "memory_type", "body"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::memory_tool::memory_save_handler,
    });

    registry.register(ToolDef {
        name: "memory_search".into(),
        description: "Search all stored memories for a keyword.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Keyword to search for in memory bodies"}
            },
            "required": ["query"]
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::memory_tool::memory_search_handler,
    });

    // SubAgent tool (Tier 2)
    registry.register(ToolDef {
        name: "subagent".into(),
        description: "Spawn an isolated sub-agent to handle a task independently. Returns a summary of the result.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "description": {"type": "string", "description": "The task for the sub-agent to complete"},
                "depth": {"type": "integer", "description": "Nesting depth (set by system, typically 0)"}
            },
            "required": ["description"]
        }),
        read_only: false,
        concurrent_safe: true,
        handler: crate::tools::subagent::subagent_handler,
    });

    // Task tools (Tier 2)
    registry.register(ToolDef {
        name: "task_create".into(),
        description: "Create a new task in the task list with optional dependencies.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Unique task identifier"},
                "content": {"type": "string", "description": "Task description"},
                "blocked_by": {"type": "array", "items": {"type": "string"}, "description": "Task IDs that must complete first"}
            },
            "required": ["id", "content"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::task::task_create_handler,
    });

    registry.register(ToolDef {
        name: "task_update".into(),
        description: "Update task status or description.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Task ID to update"},
                "status": {"type": "string", "enum": ["pending", "in_progress", "completed"]},
                "content": {"type": "string", "description": "Updated task description"}
            },
            "required": ["id"]
        }),
        read_only: false,
        concurrent_safe: false,
        handler: crate::tools::task::task_update_handler,
    });

    registry.register(ToolDef {
        name: "task_list".into(),
        description: "List all tasks and their current status.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        read_only: true,
        concurrent_safe: true,
        handler: crate::tools::task::task_list_handler,
    });
}

/// Access the global registry for read operations.
pub fn with_registry<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ToolRegistry) -> R,
{
    let guard = GLOBAL_REGISTRY.read().unwrap();
    guard.as_ref().map(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_handler(_args: Value, _root: &str) -> Result<String, String> {
        Ok("ok".into())
    }

    #[test]
    fn test_register_and_dispatch() {
        let mut reg = ToolRegistry::new("/tmp/sandbox");
        reg.register(ToolDef {
            name: "test".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
            read_only: true,
            concurrent_safe: true,
            handler: dummy_handler,
        });

        assert!(reg.has("test"));
        assert!(!reg.has("nope"));

        let result = reg.dispatch("test", serde_json::json!({}));
        assert_eq!(result.unwrap(), "ok");
    }

    #[test]
    fn test_dispatch_unknown_tool() {
        let reg = ToolRegistry::new("/tmp/sandbox");
        let result = reg.dispatch("nope", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    #[test]
    fn test_get_schemas() {
        let mut reg = ToolRegistry::new("/tmp/sandbox");
        reg.register(ToolDef {
            name: "s1".into(),
            description: "d1".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
            read_only: true,
            concurrent_safe: true,
            handler: dummy_handler,
        });

        let schemas = reg.get_schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0]["function"]["name"], "s1");
    }

    #[test]
    fn test_truncate_short_output() {
        let short = "hello".repeat(10);
        let result = truncate_output(short.clone());
        assert_eq!(result, short);
    }

    #[test]
    fn test_truncate_long_output() {
        let long = "A".repeat(40_000);
        let result = truncate_output(long);
        assert!(result.len() <= MAX_OUTPUT_BYTES + 200);
        assert!(result.contains("truncated"));
    }

    #[test]
    fn test_global_registry_init() {
        init_global_registry("/tmp/test_sandbox");
        let names = with_registry(|r| r.get_names()).unwrap();
        assert!(names.iter().any(|n| n == "read"));
        assert!(names.iter().any(|n| n == "write"));
        assert!(names.iter().any(|n| n == "bash"));
        assert!(names.iter().any(|n| n == "edit"));
        assert!(names.iter().any(|n| n == "glob"));
        assert!(names.iter().any(|n| n == "grep"));
    }
}
