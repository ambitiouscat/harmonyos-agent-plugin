use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ── Memory types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    User,
    Feedback,
    Project,
    Reference,
}

/// Frontmatter for a memory file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
}

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub frontmatter: MemoryFrontmatter,
    pub body: String,
    pub file_name: String,
}

// ── Memory store ──

pub struct MemoryStore {
    /// Directory where .memory files are stored.
    dir: PathBuf,
}

impl MemoryStore {
    pub fn new(memory_dir: &str) -> Self {
        let dir = PathBuf::from(memory_dir);
        fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    /// Ensure .memory/ directory exists.
    pub fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.dir)
            .map_err(|e| format!("Failed to create memory dir: {}", e))
    }

    /// Save a new memory file.
    pub fn save(&self, name: &str, description: &str, memory_type: MemoryType, body: &str) -> Result<String, String> {
        self.ensure_dir()?;

        let file_name = format!("{}.md", name);
        let path = self.dir.join(&file_name);

        let frontmatter = MemoryFrontmatter {
            name: name.to_string(),
            description: description.to_string(),
            memory_type,
        };

        let content = format!(
            "---\nname: {}\ndescription: {}\ntype: {}\n---\n\n{}",
            frontmatter.name,
            frontmatter.description,
            match frontmatter.memory_type {
                MemoryType::User => "user",
                MemoryType::Feedback => "feedback",
                MemoryType::Project => "project",
                MemoryType::Reference => "reference",
            },
            body
        );

        fs::write(&path, &content)
            .map_err(|e| format!("Failed to write memory: {}", e))?;

        // Update MEMORY.md index
        self.update_index(&frontmatter, &file_name)?;

        Ok(format!("Memory '{}' saved.", name))
    }

    /// Load a memory by its slug name.
    pub fn load(&self, name: &str) -> Result<MemoryEntry, String> {
        let file_name = format!("{}.md", name);
        let path = self.dir.join(&file_name);

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Memory '{}' not found: {}", name, e))?;

        Self::parse_memory_file(&content, &file_name)
    }

    /// Delete a memory by name.
    pub fn delete(&self, name: &str) -> Result<String, String> {
        let file_name = format!("{}.md", name);
        let path = self.dir.join(&file_name);

        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete memory '{}': {}", name, e))?;

        // Rebuild index
        self.rebuild_index()?;

        Ok(format!("Memory '{}' deleted.", name))
    }

    /// List all memory names from the index.
    pub fn list(&self) -> Result<Vec<String>, String> {
        let index_path = self.dir.join("MEMORY.md");
        if !index_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&index_path).unwrap_or_default();
        let names: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("- [") {
                    let start = trimmed.find('[')? + 1;
                    let end = trimmed.find(']')?;
                    Some(trimmed[start..end].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(names)
    }

    /// Search memory bodies for a keyword.
    pub fn search(&self, query: &str) -> Result<Vec<MemoryEntry>, String> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                if file_name == "MEMORY.md" {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    if content.to_lowercase().contains(&query_lower) {
                        if let Ok(memory) = Self::parse_memory_file(&content, &file_name) {
                            results.push(memory);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get full MEMORY.md index content for system prompt injection.
    pub fn get_index_content(&self) -> String {
        let index_path = self.dir.join("MEMORY.md");
        fs::read_to_string(&index_path).unwrap_or_default()
    }

    // ── Internal helpers ──

    fn parse_memory_file(content: &str, file_name: &str) -> Result<MemoryEntry, String> {
        // Parse YAML frontmatter between --- markers
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(format!("Invalid memory file format: {}", file_name));
        }

        let frontmatter_str = parts[1].trim();
        let body = parts[2].trim().to_string();

        let mut fm = HashMap::new();
        for line in frontmatter_str.lines() {
            if let Some((key, value)) = line.split_once(':') {
                fm.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        let name = fm.get("name").cloned().unwrap_or_default();
        let description = fm.get("description").cloned().unwrap_or_default();
        let memory_type = match fm.get("type").map(|s| s.as_str()) {
            Some("user") => MemoryType::User,
            Some("feedback") => MemoryType::Feedback,
            Some("project") => MemoryType::Project,
            Some("reference") => MemoryType::Reference,
            _ => MemoryType::User,
        };

        Ok(MemoryEntry {
            frontmatter: MemoryFrontmatter {
                name,
                description,
                memory_type,
            },
            body,
            file_name: file_name.to_string(),
        })
    }

    fn update_index(&self, fm: &MemoryFrontmatter, file_name: &str) -> Result<(), String> {
        let index_path = self.dir.join("MEMORY.md");
        let mut content = if index_path.exists() {
            fs::read_to_string(&index_path).unwrap_or_default()
        } else {
            String::new()
        };

        let entry = format!(
            "- [{}]({}) — {}\n",
            fm.name, file_name, fm.description
        );

        // Append if not already present
        if !content.contains(&format!("[{0}]", fm.name)) {
            content.push_str(&entry);
        }

        // Truncate to 200 lines
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 200 {
            content = lines[..200].join("\n");
            content.push('\n');
        }

        fs::write(&index_path, &content)
            .map_err(|e| format!("Failed to write index: {}", e))?;

        Ok(())
    }

    fn rebuild_index(&self) -> Result<(), String> {
        let index_path = self.dir.join("MEMORY.md");
        let mut index = String::new();

        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                if file_name == "MEMORY.md" || path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(memory) = Self::parse_memory_file(&content, &file_name) {
                        let line = format!(
                            "- [{}]({}) — {}\n",
                            memory.frontmatter.name,
                            file_name,
                            memory.frontmatter.description
                        );
                        index.push_str(&line);
                    }
                }
            }
        }

        // Truncate to 200 lines
        let lines: Vec<&str> = index.lines().collect();
        if lines.len() > 200 {
            index = lines[..200].join("\n");
            index.push('\n');
        }

        fs::write(&index_path, &index)
            .map_err(|e| format!("Failed to write index: {}", e))?;

        Ok(())
    }
}

// ── Global memory store ──

static MEMORY_STORE: std::sync::LazyLock<std::sync::RwLock<Option<MemoryStore>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(None));

pub fn init_memory_store(dir: &str) {
    let store = MemoryStore::new(dir);
    let mut guard = MEMORY_STORE.write().unwrap();
    *guard = Some(store);
}

pub fn with_memory_store<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&MemoryStore) -> R,
{
    let guard = MEMORY_STORE.read().unwrap();
    guard.as_ref().map(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> (MemoryStore, PathBuf) {
        let dir = std::env::temp_dir().join(format!("hmos_mem_{}", rand_id()));
        let _ = fs::remove_dir_all(&dir);
        let store = MemoryStore::new(dir.to_str().unwrap());
        (store, dir)
    }

    fn rand_id() -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        std::time::Instant::now().hash(&mut h);
        h.finish()
    }

    #[test]
    fn test_save_and_load() {
        let (store, _dir) = setup();
        store
            .save("test-mem", "A test memory", MemoryType::User, "Body content here.")
            .unwrap();

        let mem = store.load("test-mem").unwrap();
        assert_eq!(mem.frontmatter.name, "test-mem");
        assert_eq!(mem.body, "Body content here.");
    }

    #[test]
    fn test_list_and_delete() {
        let (store, _dir) = setup();
        store
            .save("m1", "Memory 1", MemoryType::User, "body 1")
            .unwrap();
        store
            .save("m2", "Memory 2", MemoryType::Project, "body 2")
            .unwrap();

        let names = store.list().unwrap();
        assert_eq!(names.len(), 2);

        store.delete("m1").unwrap();
        let names = store.list().unwrap();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "m2");
    }

    #[test]
    fn test_search() {
        let (store, _dir) = setup();
        store
            .save("mem-a", "Memory A", MemoryType::User, "This is about Rust coding.")
            .unwrap();
        store
            .save("mem-b", "Memory B", MemoryType::Project, "This is about HarmonyOS.")
            .unwrap();

        let results = store.search("Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].frontmatter.name, "mem-a");

        let results = store.search("HarmonyOS").unwrap();
        assert_eq!(results.len(), 1);

        let results = store.search("nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_index_content() {
        let (store, _dir) = setup();
        store
            .save("idx-test", "Index test memory", MemoryType::Reference, "content")
            .unwrap();

        let idx = store.get_index_content();
        assert!(idx.contains("idx-test"));
        assert!(idx.contains("Index test memory"));
    }
}
