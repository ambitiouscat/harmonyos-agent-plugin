use crate::agent::skills::{Skill, SkillCategory, SkillContext, SkillParam};
use std::collections::HashMap;
use std::path::Path;

/// File-system based skill loader.
/// Scans a directory tree for `SKILL.md` files and parses them into `Box<dyn Skill>`.
pub struct FileSystemSkillLoader;

/// A skill loaded from a SKILL.md file.
struct FileSkill {
    name: String,
    description: String,
    category: SkillCategory,
    params: Vec<SkillParam>,
    body: String,
}

impl Skill for FileSkill {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn category(&self) -> SkillCategory {
        self.category.clone()
    }
    fn params(&self) -> Vec<SkillParam> {
        self.params.clone()
    }
    fn execute(&self, _ctx: &SkillContext) -> Result<String, String> {
        // File-based skills return their full body content for context injection
        Ok(self.body.clone())
    }
}

impl FileSystemSkillLoader {
    /// Scan a directory for SKILL.md files and load them as dynamic skills.
    /// Each subdirectory containing a `SKILL.md` becomes one skill.
    pub fn scan(dir: &Path) -> Result<Vec<Box<dyn Skill>>, String> {
        let mut skills: Vec<Box<dyn Skill>> = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(skills);
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read skills dir '{}': {}", dir.display(), e))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest = path.join("SKILL.md");
            if !manifest.exists() {
                continue;
            }

            let content = match std::fs::read_to_string(&manifest) {
                Ok(c) => c,
                Err(_) => continue,
            };

            match Self::parse_skill_file(&content, &path) {
                Ok(skill) => skills.push(Box::new(skill)),
                Err(e) => {
                    eprintln!(
                        "[skill_loader] Warning: failed to parse '{}': {}",
                        manifest.display(),
                        e
                    );
                }
            }
        }

        Ok(skills)
    }

    /// Parse a SKILL.md file's YAML frontmatter + Markdown body.
    fn parse_skill_file(content: &str, dir: &Path) -> Result<FileSkill, String> {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err("Missing YAML frontmatter (expected --- delimiters)".into());
        }

        let frontmatter_str = parts[1].trim();
        let body = parts[2].trim().to_string();

        let mut fm: HashMap<String, String> = HashMap::new();
        for line in frontmatter_str.lines() {
            if let Some((key, value)) = line.split_once(':') {
                fm.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        let name = fm
            .get("name")
            .cloned()
            .unwrap_or_else(|| dir.file_name().unwrap_or_default().to_string_lossy().to_string());
        let description = fm
            .get("description")
            .cloned()
            .unwrap_or_else(|| "No description".to_string());
        let category = fm
            .get("category")
            .map(|s| SkillCategory::from_str(s))
            .unwrap_or(SkillCategory::Other);

        // Parse params from frontmatter (params: name:type:desc:required, ...)
        let params: Vec<SkillParam> = fm
            .get("params")
            .map(|v| {
                v.split(',')
                    .filter_map(|p| {
                        let parts: Vec<&str> = p.trim().split(':').collect();
                        if parts.len() >= 4 {
                            Some(SkillParam {
                                name: parts[0].trim().to_string(),
                                param_type: parts[1].trim().to_string(),
                                description: parts[2].trim().to_string(),
                                required: parts[3].trim().eq_ignore_ascii_case("required"),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(FileSkill {
            name,
            description,
            category,
            params,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::platform::HostCapabilities;
    use std::fs;
    use std::path::Path;

    /// Test stub — a HostCapabilities impl that rejects all IO.
    struct NoopHost;
    impl HostCapabilities for NoopHost {
        fn read_file(&self, _: &Path) -> Result<String, String> { Err("noop".into()) }
        fn write_file(&self, _: &Path, _: &str) -> Result<(), String> { Err("noop".into()) }
        fn http_post(&self, _: &str, _: &str) -> Result<String, String> { Err("noop".into()) }
        fn execute_command(&self, _: &str) -> Result<String, String> { Err("noop".into()) }
        fn spawn_task(&self, _: Box<dyn FnOnce() + Send>) {}
    }

    #[test]
    fn test_scan_empty_dir() {
        let dir = std::env::temp_dir().join("hmos_skills_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let skills = FileSystemSkillLoader::scan(&dir).unwrap();
        assert!(skills.is_empty());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_with_skill() {
        let dir = std::env::temp_dir().join("hmos_skills_test");
        let _ = fs::remove_dir_all(&dir);
        let skill_dir = dir.join("git-helper");
        fs::create_dir_all(&skill_dir).unwrap();

        let skill_md = r#"---
name: git-helper
description: Help with common Git operations
category: utility
params: action:string:Git action to perform:required, branch:string:Target branch:optional
---

## git-helper

Assists with common Git operations like commit, push, branch management.
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md).unwrap();

        let skills = FileSystemSkillLoader::scan(&dir).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name(), "git-helper");
        assert_eq!(skills[0].category(), SkillCategory::Utility);
        assert_eq!(skills[0].params().len(), 2);
        assert!(skills[0]
            .description()
            .contains("common Git operations"));

        // execute returns body content
        let ctx = SkillContext {
            working_dir: std::path::PathBuf::from("."),
            args: HashMap::new(),
            host: &NoopHost,
        };
        let result = skills[0].execute(&ctx).unwrap();
        assert!(result.contains("git-helper"));
        assert!(result.contains("Assists with common Git operations"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let dir = std::env::temp_dir().join("hmos_skills_nonexistent_12345");
        let skills = FileSystemSkillLoader::scan(&dir).unwrap();
        assert!(skills.is_empty());
    }
}
