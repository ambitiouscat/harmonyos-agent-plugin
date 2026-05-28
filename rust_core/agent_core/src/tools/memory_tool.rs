use serde_json::Value;

use crate::agent::memory::{with_memory_store, MemoryType};

pub fn memory_save_handler(args: Value, _sandbox_root: &str) -> Result<String, String> {
    let name = args["name"].as_str().ok_or("Missing 'name'")?;
    let description = args["description"].as_str().ok_or("Missing 'description'")?;
    let mem_type = match args["memory_type"].as_str().unwrap_or("knowledge") {
        "session" => MemoryType::Session,
        "conversation" => MemoryType::Conversation,
        "knowledge" => MemoryType::Knowledge,
        "preference" => MemoryType::Preference,
        "task" => MemoryType::Task,
        "error" => MemoryType::Error,
        "insight" => MemoryType::Insight,
        // Legacy aliases
        "user" => MemoryType::Preference,
        "feedback" => MemoryType::Insight,
        "project" => MemoryType::Knowledge,
        "reference" => MemoryType::Knowledge,
        other => return Err(format!("Invalid memory type '{}'. Valid types: session, conversation, knowledge, preference, task, error, insight", other)),
    };
    let body = args["body"].as_str().ok_or("Missing 'body'")?;

    with_memory_store(|store| store.save(name, description, mem_type, body))
        .ok_or("Memory store not initialized")?
}

pub fn memory_search_handler(args: Value, _sandbox_root: &str) -> Result<String, String> {
    let query = args["query"].as_str().ok_or("Missing 'query'")?;
    let results = with_memory_store(|store| store.search(query))
        .ok_or("Memory store not initialized")??;

    if results.is_empty() {
        Ok("No matching memories found.".into())
    } else {
        let lines: Vec<String> = results
            .iter()
            .map(|m| format!("- {}: {}", m.frontmatter.name, m.frontmatter.description))
            .collect();
        Ok(lines.join("\n"))
    }
}

pub fn memory_list_handler(_args: Value, _sandbox_root: &str) -> Result<String, String> {
    let names = with_memory_store(|store| store.list())
        .ok_or("Memory store not initialized")??;

    if names.is_empty() {
        Ok("No memories stored.".into())
    } else {
        Ok(names.join("\n"))
    }
}
