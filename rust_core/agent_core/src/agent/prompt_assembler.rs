use std::sync::RwLock;

/// Cache key and result for json.dumps-style caching.
#[derive(Clone, PartialEq, Eq, Hash)]
struct PromptCacheKey {
    tools_hash: u64,
    skills_len: usize,
    memory_len: usize,
    claude_md_len: usize,
}

static PROMPT_CACHE: RwLock<Option<(PromptCacheKey, String)>> = RwLock::new(None);

/// Assemble the system prompt from all available context sources.
pub fn assemble_system_prompt(
    workspace_root: &str,
) -> String {
    let mut sections: Vec<String> = Vec::new();

    // ── Identity ──
    sections.push(IDENTITY_SECTION.to_string());

    // ── Workspace ──
    sections.push(format!(
        "## Workspace\n\nCurrent working directory: `{}`\n",
        workspace_root
    ));

    // ── Tools catalog (from ToolRegistry) ──
    if let Some(tools_str) = build_tools_section() {
        sections.push(tools_str);
    }

    // ── Skills catalog (from SkillsRegistry) ──
    let skills_section = build_skills_section();
    sections.push(skills_section);

    // ── Memory index (from MemoryStore) ──
    if let Some(mem_str) = build_memory_section() {
        sections.push(mem_str);
    }

    // ── CLAUDE.md ──
    if let Some(claude_md) = load_claude_md(workspace_root) {
        sections.push(claude_md);
    }

    sections.join("\n\n")
}

/// Get system prompt with simple caching (reassembles if context changed).
pub fn get_system_prompt(workspace_root: &str) -> String {
    let prompt = assemble_system_prompt(workspace_root);

    // Simple cache: check if prompt length matches
    if let Ok(guard) = PROMPT_CACHE.read() {
        if let Some((_, cached)) = guard.as_ref() {
            if cached.len() == prompt.len() {
                return cached.clone();
            }
        }
    }

    if let Ok(mut guard) = PROMPT_CACHE.write() {
        let key = PromptCacheKey {
            tools_hash: 0,
            skills_len: prompt.len(),
            memory_len: 0,
            claude_md_len: 0,
        };
        *guard = Some((key, prompt.clone()));
    }

    prompt
}

// ── Section builders ──

const IDENTITY_SECTION: &str = "\
## Identity

You are HmosAgent, an AI coding agent that runs inside a HarmonyOS app. \
You have access to tools for reading, writing, editing files, executing bash commands, \
searching code, managing tasks, spawning sub-agents, loading skills, and persisting memories.

You operate in a sandboxed workspace. All file paths are relative to the workspace root. \
Use tools to complete user requests. Be thorough and precise. \
When you encounter complex multi-step tasks, use the task tools to track your progress.";

fn build_tools_section() -> Option<String> {
    let names = crate::agent::tool_registry::with_registry(|r| r.get_names())?;
    let tools_list = names
        .iter()
        .map(|n| format!("- `{}`", n))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("## Available Tools\n\nThe following tools are registered:\n\n{}", tools_list))
}

fn build_skills_section() -> String {
    let skills = crate::agent::skills::SKILLS.read().unwrap();
    let catalog = skills.get_catalog();
    if catalog.is_empty() {
        return String::new();
    }
    let lines: Vec<String> = catalog
        .iter()
        .map(|s| format!("- `{}` ({}): {}", s.name, s.category, s.description))
        .collect();
    format!("## Available Skills\n\n{}", lines.join("\n"))
}

fn build_memory_section() -> Option<String> {
    let idx = crate::agent::memory::with_memory_store(|store| store.get_index_content())?;
    if idx.trim().is_empty() {
        return None;
    }
    Some(format!(
        "## Persistent Memory\n\nThe following memories are stored across sessions:\n\n{}",
        idx
    ))
}

fn load_claude_md(workspace_root: &str) -> Option<String> {
    let path = std::path::PathBuf::from(workspace_root).join("CLAUDE.md");
    let content = std::fs::read_to_string(&path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(format!(
        "## Project Instructions (CLAUDE.md)\n\n{}",
        content
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_returns_identity() {
        crate::agent::tool_registry::init_global_registry("/tmp/test");
        let prompt = assemble_system_prompt("/tmp/test");
        assert!(prompt.contains("HmosAgent"));
    }

    #[test]
    fn test_assemble_includes_workspace() {
        let prompt = assemble_system_prompt("/custom/path");
        assert!(prompt.contains("/custom/path"));
    }

    #[test]
    fn test_get_system_prompt_non_empty() {
        let p = get_system_prompt("/tmp/some_path");
        assert!(!p.is_empty());
        assert!(p.contains("HmosAgent"));
    }

    #[test]
    fn test_skills_section() {
        let section = build_skills_section();
        assert!(section.contains("/search"));
        assert!(section.contains("/goal"));
    }
}
