use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

// ── SkillCategory (V2: strong-typed enum) ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillCategory {
    Process,
    Implementation,
    Platform,
    Review,
    Utility,
    Documentation,
    Other,
    // Legacy string aliases preserved for backward compat
    #[serde(alias = "Code")]
    #[serde(alias = "code")]
    Code,
    #[serde(alias = "Agent")]
    #[serde(alias = "agent")]
    Agent,
    #[serde(alias = "File")]
    #[serde(alias = "file")]
    File,
    #[serde(alias = "Session")]
    #[serde(alias = "session")]
    Session,
}

impl SkillCategory {
    pub fn as_str(&self) -> &str {
        match self {
            SkillCategory::Process => "process",
            SkillCategory::Implementation => "implementation",
            SkillCategory::Platform => "platform",
            SkillCategory::Review => "review",
            SkillCategory::Utility => "utility",
            SkillCategory::Documentation => "documentation",
            SkillCategory::Other => "other",
            SkillCategory::Code => "code",
            SkillCategory::Agent => "agent",
            SkillCategory::File => "file",
            SkillCategory::Session => "session",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "process" => SkillCategory::Process,
            "implementation" => SkillCategory::Implementation,
            "platform" => SkillCategory::Platform,
            "review" => SkillCategory::Review,
            "utility" => SkillCategory::Utility,
            "documentation" => SkillCategory::Documentation,
            "other" => SkillCategory::Other,
            "code" | "Code" => SkillCategory::Code,
            "agent" | "Agent" => SkillCategory::Agent,
            "file" | "File" => SkillCategory::File,
            "session" | "Session" => SkillCategory::Session,
            _ => SkillCategory::Other,
        }
    }
}

// ── Lightweight catalog entry for system prompt ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub category: String,
}

// ── Skill parameters ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

// ── SkillDef (existing fn-pointer-based definition for built-in skills) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub description: String,
    pub category: String,
    pub params: Vec<SkillParam>,
}

// ── Skill trait (V2: trait-based for dynamic/file-loaded skills) ──

/// Synchronous skill trait — object-safe.
/// Implementations: built-in skill wrappers, file-loaded skills, plugin skills.
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> SkillCategory;
    fn params(&self) -> Vec<SkillParam>;
    fn execute(&self, ctx: &SkillContext) -> Result<String, String>;
}

/// Context passed to Skill::execute.
pub struct SkillContext<'a> {
    pub working_dir: PathBuf,
    pub args: HashMap<String, String>,
    pub host: &'a (dyn crate::agent::platform::HostCapabilities + 'a),
}

// ── SkillsRegistry (V2: supports both SkillDef and Box<dyn Skill>) ──

pub struct SkillsRegistry {
    /// Built-in skills registered via SkillDef (fn-pointer based, backward compat).
    defs: HashMap<String, SkillDef>,
    /// Dynamic skills registered via the Skill trait (file-loaded, plugin-loaded).
    dynamic: HashMap<String, Box<dyn Skill>>,
}

impl SkillsRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            defs: HashMap::new(),
            dynamic: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    /// Register a built-in SkillDef.
    pub fn register_def(&mut self, def: SkillDef) {
        self.defs.insert(def.name.clone(), def);
    }

    /// Register a dynamic Skill trait object.
    pub fn register_dynamic(&mut self, skill: Box<dyn Skill>) {
        self.dynamic.insert(skill.name().to_string(), skill);
    }

    /// Legacy alias for register_def.
    pub fn register(&mut self, def: SkillDef) {
        self.register_def(def);
    }

    /// Get all skill names (built-in + dynamic).
    pub fn get_all(&self) -> Vec<SkillDef> {
        let mut all: Vec<SkillDef> = self.defs.values().cloned().collect();
        for skill in self.dynamic.values() {
            all.push(SkillDef {
                name: skill.name().to_string(),
                description: skill.description().to_string(),
                category: skill.category().as_str().to_string(),
                params: skill.params(),
            });
        }
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    }

    /// Look up a skill by name.
    pub fn get_by_name(&self, name: &str) -> Option<SkillDef> {
        if let Some(def) = self.defs.get(name) {
            return Some(def.clone());
        }
        self.dynamic.get(name).map(|s| SkillDef {
            name: s.name().to_string(),
            description: s.description().to_string(),
            category: s.category().as_str().to_string(),
            params: s.params(),
        })
    }

    /// Look up a dynamic Skill trait object by name.
    pub fn get_dynamic(&self, name: &str) -> Option<&dyn Skill> {
        self.dynamic.get(name).map(|b| b.as_ref())
    }

    pub fn match_prefix(&self, prefix: &str) -> Vec<SkillDef> {
        let lower = prefix.to_lowercase();
        let mut results: Vec<SkillDef> = self
            .defs
            .values()
            .filter(|s| s.name.to_lowercase().starts_with(&lower))
            .cloned()
            .collect();
        for skill in self.dynamic.values() {
            let n = skill.name().to_lowercase();
            if n.starts_with(&lower) {
                results.push(SkillDef {
                    name: skill.name().to_string(),
                    description: skill.description().to_string(),
                    category: skill.category().as_str().to_string(),
                    params: skill.params(),
                });
            }
        }
        results
    }

    pub fn get_catalog(&self) -> Vec<SkillMeta> {
        let mut catalog: Vec<SkillMeta> = self
            .defs
            .values()
            .map(|s| SkillMeta {
                name: s.name.clone(),
                description: s.description.clone(),
                category: s.category.clone(),
            })
            .collect();
        for skill in self.dynamic.values() {
            catalog.push(SkillMeta {
                name: skill.name().to_string(),
                description: skill.description().to_string(),
                category: skill.category().as_str().to_string(),
            });
        }
        catalog
    }

    /// Load full skill content — for Def-based skills, returns formatted params;
    /// for dynamic skills, returns the skill name + description.
    pub fn load_skill_detail(&self, name: &str) -> Option<String> {
        // Try dynamic first
        if let Some(skill) = self.dynamic.get(name) {
            let params_str: Vec<String> = skill
                .params()
                .iter()
                .map(|p| {
                    format!(
                        "  - {} ({}, {}): {}",
                        p.name,
                        p.param_type,
                        if p.required { "required" } else { "optional" },
                        p.description
                    )
                })
                .collect();
            return Some(format!(
                "## {}\n\n**Category:** {}\n\n{}\n\n### Parameters\n\n{}",
                skill.name(),
                skill.category().as_str(),
                skill.description(),
                if params_str.is_empty() {
                    "  (no parameters)".into()
                } else {
                    params_str.join("\n")
                }
            ));
        }

        // Fall back to Def
        self.defs.get(name).map(|s| {
            let params_str: Vec<String> = s
                .params
                .iter()
                .map(|p| {
                    format!(
                        "  - {} ({}, {}): {}",
                        p.name,
                        p.param_type,
                        if p.required { "required" } else { "optional" },
                        p.description
                    )
                })
                .collect();
            format!(
                "## {}\n\n**Category:** {}\n\n{}\n\n### Parameters\n\n{}",
                s.name,
                s.category,
                s.description,
                if params_str.is_empty() {
                    "  (no parameters)".into()
                } else {
                    params_str.join("\n")
                }
            )
        })
    }

    /// Legacy alias.
    pub fn load_skill(&self, name: &str) -> Option<String> {
        self.load_skill_detail(name)
    }

    pub fn fuzzy_match(&self, query: &str) -> Vec<SkillDef> {
        let lower = query.to_lowercase();
        let mut results: Vec<(usize, SkillDef)> = Vec::new();

        for s in self.defs.values() {
            let name_lower = s.name.to_lowercase();
            if name_lower.contains(&lower) {
                let score = if name_lower.starts_with(&lower) {
                    0
                } else {
                    name_lower.find(&lower).unwrap_or(usize::MAX)
                };
                results.push((score, s.clone()));
            }
        }

        for skill in self.dynamic.values() {
            let name_lower = skill.name().to_lowercase();
            if name_lower.contains(&lower) {
                let score = if name_lower.starts_with(&lower) {
                    0
                } else {
                    name_lower.find(&lower).unwrap_or(usize::MAX)
                };
                results.push((
                    score,
                    SkillDef {
                        name: skill.name().to_string(),
                        description: skill.description().to_string(),
                        category: skill.category().as_str().to_string(),
                        params: skill.params(),
                    },
                ));
            }
        }

        results.sort_by_key(|(score, _)| *score);
        results.into_iter().map(|(_, s)| s).collect()
    }

    // ── Built-in registration ──

    fn register_builtins(&mut self) {
        self.register_def(SkillDef {
            name: "/search".into(),
            description: "Search codebase with ripgrep for a pattern".into(),
            category: "code".into(),
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
                    description: "Directory to search in (default: workspace root)".into(),
                    required: false,
                },
            ],
        });

        self.register_def(SkillDef {
            name: "/scan".into(),
            description: "Scan directory and build RAG index for semantic search".into(),
            category: "code".into(),
            params: vec![SkillParam {
                name: "path".into(),
                param_type: "string".into(),
                description: "Directory to scan for documents".into(),
                required: true,
            }],
        });

        self.register_def(SkillDef {
            name: "/goal".into(),
            description: "Set a goal or task for the agent to work towards".into(),
            category: "agent".into(),
            params: vec![SkillParam {
                name: "description".into(),
                param_type: "string".into(),
                description: "Description of the goal".into(),
                required: true,
            }],
        });

        self.register_def(SkillDef {
            name: "/file".into(),
            description: "Read or write files in the sandbox workspace".into(),
            category: "file".into(),
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

        self.register_def(SkillDef {
            name: "/session".into(),
            description: "Manage conversation sessions (list/new/switch/delete)".into(),
            category: "session".into(),
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
}

impl Default for SkillsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global skills registry singleton.
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

    // ── V2 tests ──

    /// A sample dynamic skill for testing.
    struct TestSkill {
        name: String,
        description: String,
        category: SkillCategory,
    }

    impl Skill for TestSkill {
        fn name(&self) -> &str { &self.name }
        fn description(&self) -> &str { &self.description }
        fn category(&self) -> SkillCategory { self.category.clone() }
        fn params(&self) -> Vec<SkillParam> { vec![] }

        fn execute(&self, ctx: &SkillContext) -> Result<String, String> {
            Ok(format!("Executed {} in {:?}", self.name, ctx.working_dir))
        }
    }

    #[test]
    fn test_dynamic_skill_registration() {
        let mut reg = SkillsRegistry::new();
        let dyn_skill = Box::new(TestSkill {
            name: "/test-dyn".into(),
            description: "A dynamic test skill".into(),
            category: SkillCategory::Utility,
        });
        reg.register_dynamic(dyn_skill);

        assert!(reg.get_by_name("/test-dyn").is_some());
        assert!(reg.get_dynamic("/test-dyn").is_some());
        assert_eq!(
            reg.get_dynamic("/test-dyn").unwrap().category(),
            SkillCategory::Utility
        );
    }

    #[test]
    fn test_dynamic_skill_in_catalog() {
        let mut reg = SkillsRegistry::new();
        reg.register_dynamic(Box::new(TestSkill {
            name: "/dyn-catalog".into(),
            description: "Catalog test".into(),
            category: SkillCategory::Process,
        }));

        let catalog = reg.get_catalog();
        assert!(catalog.iter().any(|s| s.name == "/dyn-catalog"));
    }

    #[test]
    fn test_skill_category_roundtrip() {
        let cats = vec![
            ("process", SkillCategory::Process),
            ("implementation", SkillCategory::Implementation),
            ("platform", SkillCategory::Platform),
            ("review", SkillCategory::Review),
            ("utility", SkillCategory::Utility),
            ("documentation", SkillCategory::Documentation),
            ("other", SkillCategory::Other),
            ("code", SkillCategory::Code), // legacy
        ];
        for (s, expected) in &cats {
            assert_eq!(SkillCategory::from_str(s), *expected);
            assert_eq!(expected.as_str(), *s);
        }
    }
}
