use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub description: String,
    pub category: String,
    pub params: Vec<SkillParam>,
}

pub struct SkillsRegistry {
    skills: HashMap<String, SkillDef>,
}

impl SkillsRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            skills: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    fn register_builtins(&mut self) {
        self.register(SkillDef {
            name: "/search".into(),
            description: "Search codebase with ripgrep for a pattern".into(),
            category: "Code".into(),
            params: vec![
                SkillParam {
                    name: "pattern".into(),
                    param_type: "string".into(),
                    description: "Regex pattern to search for".into(),
                    required: true,
                },
                SkillParam {
                    name: "path".into(),
                    param_type: "string".into(),
                    description: "Directory to search in (default: workspace root)"
                        .into(),
                    required: false,
                },
            ],
        });

        self.register(SkillDef {
            name: "/scan".into(),
            description: "Scan directory and build RAG index for semantic search".into(),
            category: "Code".into(),
            params: vec![SkillParam {
                name: "path".into(),
                param_type: "string".into(),
                description: "Directory to scan for documents".into(),
                required: true,
            }],
        });

        self.register(SkillDef {
            name: "/goal".into(),
            description: "Set a goal or task for the agent to work towards".into(),
            category: "Agent".into(),
            params: vec![SkillParam {
                name: "description".into(),
                param_type: "string".into(),
                description: "Description of the goal".into(),
                required: true,
            }],
        });

        self.register(SkillDef {
            name: "/file".into(),
            description: "Read or write files in the sandbox workspace".into(),
            category: "File".into(),
            params: vec![
                SkillParam {
                    name: "action".into(),
                    param_type: "string".into(),
                    description: "read or write".into(),
                    required: true,
                },
                SkillParam {
                    name: "path".into(),
                    param_type: "string".into(),
                    description: "Relative path in workspace".into(),
                    required: true,
                },
                SkillParam {
                    name: "content".into(),
                    param_type: "string".into(),
                    description: "Content to write (required for write action)".into(),
                    required: false,
                },
            ],
        });

        self.register(SkillDef {
            name: "/session".into(),
            description: "Manage conversation sessions (list/new/switch/delete)".into(),
            category: "Session".into(),
            params: vec![
                SkillParam {
                    name: "action".into(),
                    param_type: "string".into(),
                    description: "list, new, switch, or delete".into(),
                    required: true,
                },
                SkillParam {
                    name: "name".into(),
                    param_type: "string".into(),
                    description: "Session name or ID".into(),
                    required: false,
                },
            ],
        });
    }

    pub fn register(&mut self, skill: SkillDef) {
        self.skills.insert(skill.name.clone(), skill);
    }

    pub fn get_all(&self) -> Vec<SkillDef> {
        let mut skills: Vec<SkillDef> = self.skills.values().cloned().collect();
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        skills
    }

    pub fn get_by_name(&self, name: &str) -> Option<&SkillDef> {
        self.skills.get(name)
    }

    pub fn match_prefix(&self, prefix: &str) -> Vec<SkillDef> {
        let lower = prefix.to_lowercase();
        self.skills
            .values()
            .filter(|s| s.name.to_lowercase().starts_with(&lower))
            .cloned()
            .collect()
    }

    pub fn fuzzy_match(&self, query: &str) -> Vec<SkillDef> {
        let lower = query.to_lowercase();
        let mut results: Vec<(usize, SkillDef)> = self
            .skills
            .values()
            .filter_map(|s| {
                let name_lower = s.name.to_lowercase();
                if name_lower.contains(&lower) {
                    // Score: exact prefix match = 0, otherwise distance-based
                    let score = if name_lower.starts_with(&lower) {
                        0
                    } else {
                        name_lower.find(&lower).unwrap_or(usize::MAX)
                    };
                    Some((score, s.clone()))
                } else {
                    None
                }
            })
            .collect();
        results.sort_by_key(|(score, _)| *score);
        results.into_iter().map(|(_, s)| s).collect()
    }
}

impl Default for SkillsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global skills registry singleton — lazily initialized with builtin skills
pub static SKILLS: LazyLock<RwLock<SkillsRegistry>> =
    LazyLock::new(|| RwLock::new(SkillsRegistry::new()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_skills_registered() {
        let reg = SkillsRegistry::new();
        let all = reg.get_all();
        assert!(all.len() >= 3, "expected at least 3 builtin skills");
        assert!(reg.get_by_name("/search").is_some());
        assert!(reg.get_by_name("/scan").is_some());
        assert!(reg.get_by_name("/goal").is_some());
    }

    #[test]
    fn test_prefix_match() {
        let reg = SkillsRegistry::new();
        let results = reg.match_prefix("/s");
        assert!(results.iter().any(|s| s.name == "/search"));
        assert!(results.iter().any(|s| s.name == "/scan"));
    }

    #[test]
    fn test_fuzzy_match() {
        let reg = SkillsRegistry::new();
        let results = reg.fuzzy_match("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "/search");
    }

    #[test]
    fn test_fuzzy_match_partial() {
        let reg = SkillsRegistry::new();
        let results = reg.fuzzy_match("sea");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "/search");
    }

    #[test]
    fn test_fuzzy_match_finds_multiple() {
        let reg = SkillsRegistry::new();
        let results = reg.fuzzy_match("s");
        assert!(results.len() >= 2);
    }
}
