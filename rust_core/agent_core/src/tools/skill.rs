use serde_json::Value;

/// skill_load tool handler — returns full skill content for context injection.
pub fn skill_load_handler(args: Value, _sandbox_root: &str) -> Result<String, String> {
    let name = args["name"]
        .as_str()
        .ok_or_else(|| "Missing 'name' parameter".to_string())?;

    let skills = crate::agent::skills::SKILLS.read().unwrap();
    match skills.load_skill(name) {
        Some(content) => Ok(content),
        None => {
            // Try fuzzy match for helpful error
            let matches = skills.fuzzy_match(name);
            if matches.is_empty() {
                Err(format!("Skill '{}' not found.", name))
            } else {
                let suggestions: Vec<String> = matches.iter().map(|s| s.name.clone()).collect();
                Err(format!(
                    "Skill '{}' not found. Did you mean: {}?",
                    name,
                    suggestions.join(", ")
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_load_found() {
        let result = skill_load_handler(
            serde_json::json!({"name": "/search"}),
            "",
        );
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("/search"));
        assert!(content.contains("Search codebase"));
    }

    #[test]
    fn test_skill_load_not_found() {
        let result = skill_load_handler(
            serde_json::json!({"name": "/nonexistent"}),
            "",
        );
        assert!(result.is_err());
    }
}
