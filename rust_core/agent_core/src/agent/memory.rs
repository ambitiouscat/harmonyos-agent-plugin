use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ── Memory types (V2: 7 variants) ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    Session,
    Conversation,
    Knowledge,
    Preference,
    Task,
    Error,
    Insight,
    // Legacy aliases — preserved for backward compat with existing .md files
    #[serde(alias = "user")]
    User,
    #[serde(alias = "feedback")]
    Feedback,
    #[serde(alias = "project")]
    Project,
    #[serde(alias = "reference")]
    Reference,
}

impl MemoryType {
    /// Resolve legacy type names to their V2 equivalents.
    pub fn resolve(legacy: &str) -> Self {
        match legacy {
            "user" => MemoryType::Preference,
            "feedback" => MemoryType::Insight,
            "project" => MemoryType::Knowledge,
            "reference" => MemoryType::Knowledge,
            "session" => MemoryType::Session,
            "conversation" => MemoryType::Conversation,
            "knowledge" => MemoryType::Knowledge,
            "preference" => MemoryType::Preference,
            "task" => MemoryType::Task,
            "error" => MemoryType::Error,
            "insight" => MemoryType::Insight,
            _ => MemoryType::Knowledge,
        }
    }

    /// Canonical string representation for file serialization.
    pub fn as_str(&self) -> &str {
        match self {
            MemoryType::Session => "session",
            MemoryType::Conversation => "conversation",
            MemoryType::Knowledge => "knowledge",
            MemoryType::Preference => "preference",
            MemoryType::Task => "task",
            MemoryType::Error => "error",
            MemoryType::Insight => "insight",
            // Legacy aliases — serialize to their canonical V2 names
            MemoryType::User => "preference",
            MemoryType::Feedback => "insight",
            MemoryType::Project => "knowledge",
            MemoryType::Reference => "knowledge",
        }
    }
}

/// Frontmatter for a memory file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
}

/// A single memory entry (V2: extended fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub frontmatter: MemoryFrontmatter,
    pub body: String,
    pub file_name: String,
    /// Relevance score 0.0–1.0, defaults to 0.5.
    #[serde(default = "default_importance")]
    pub importance: f32,
    /// User-defined tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Arbitrary key-value metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Optional embedding vector for future vector search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

fn default_importance() -> f32 {
    0.5
}

// ── MemoryStorage trait ──

/// Platform-agnostic persistence for memory entries.
/// Implementations: FileSystemMemoryStore (HarmonyOS/Desktop), WasmMemoryStore (WASM).
pub trait MemoryStorage: Send + Sync {
    fn save(&self, name: &str, description: &str, memory_type: MemoryType, body: &str) -> Result<String, String>;
    fn load(&self, name: &str) -> Result<MemoryEntry, String>;
    fn delete(&self, name: &str) -> Result<String, String>;
    fn list(&self) -> Result<Vec<String>, String>;
    fn search(&self, query: &str) -> Result<Vec<MemoryEntry>, String>;
    fn get_index_content(&self) -> String;
}

// ── FileSystemMemoryStore (HarmonyOS / Desktop) ──

pub struct FileSystemMemoryStore {
    dir: PathBuf,
}

impl FileSystemMemoryStore {
    pub fn new(memory_dir: &str) -> Self {
        let dir = PathBuf::from(memory_dir);
        fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.dir)
            .map_err(|e| format!("Failed to create memory dir: {}", e))
    }

    fn parse_memory_file(content: &str, file_name: &str) -> Result<MemoryEntry, String> {
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
        let memory_type = fm
            .get("type")
            .map(|s| MemoryType::resolve(s))
            .unwrap_or(MemoryType::Knowledge);
        let importance: f32 = fm
            .get("importance")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.5)
            .max(0.0)
            .min(1.0);
        let tags: Vec<String> = fm
            .get("tags")
            .map(|v| v.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();
        let metadata: HashMap<String, String> = fm
            .iter()
            .filter(|(k, _)| {
                !matches!(
                    k.as_str(),
                    "name" | "description" | "type" | "importance" | "tags" | "embedding"
                )
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(MemoryEntry {
            frontmatter: MemoryFrontmatter {
                name,
                description,
                memory_type,
            },
            body,
            file_name: file_name.to_string(),
            importance,
            tags,
            metadata,
            embedding: None,
        })
    }

    fn update_index(&self, fm: &MemoryFrontmatter, file_name: &str) -> Result<(), String> {
        let index_path = self.dir.join("MEMORY.md");
        let mut content = if index_path.exists() {
            fs::read_to_string(&index_path).unwrap_or_default()
        } else {
            String::new()
        };

        let entry = format!("- [{}]({}) — {}\n", fm.name, file_name, fm.description);

        if !content.contains(&format!("[{0}]", fm.name)) {
            content.push_str(&entry);
        }

        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 200 {
            let trimmed: String = lines[..200].join("\n");
            fs::write(&index_path, trimmed + "\n")
        } else {
            fs::write(&index_path, &content)
        }
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
                if file_name == "MEMORY.md"
                    || path.extension().and_then(|e| e.to_str()) != Some("md")
                {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(memory) = Self::parse_memory_file(&content, &file_name) {
                        let line = format!(
                            "- [{}]({}) — {}\n",
                            memory.frontmatter.name, file_name, memory.frontmatter.description
                        );
                        index.push_str(&line);
                    }
                }
            }
        }

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

impl MemoryStorage for FileSystemMemoryStore {
    fn save(
        &self,
        name: &str,
        description: &str,
        memory_type: MemoryType,
        body: &str,
    ) -> Result<String, String> {
        self.ensure_dir()?;

        let file_name = format!("{}.md", name);
        let path = self.dir.join(&file_name);

        let frontmatter = MemoryFrontmatter {
            name: name.to_string(),
            description: description.to_string(),
            memory_type: memory_type.clone(),
        };

        let content = format!(
            "---\nname: {}\ndescription: {}\ntype: {}\n---\n\n{}",
            frontmatter.name,
            frontmatter.description,
            memory_type.as_str(),
            body
        );

        fs::write(&path, &content)
            .map_err(|e| format!("Failed to write memory: {}", e))?;

        self.update_index(&frontmatter, &file_name)?;

        Ok(format!("Memory '{}' saved.", name))
    }

    fn load(&self, name: &str) -> Result<MemoryEntry, String> {
        let file_name = format!("{}.md", name);
        let path = self.dir.join(&file_name);

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Memory '{}' not found: {}", name, e))?;

        Self::parse_memory_file(&content, &file_name)
    }

    fn delete(&self, name: &str) -> Result<String, String> {
        let file_name = format!("{}.md", name);
        let path = self.dir.join(&file_name);

        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete memory '{}': {}", name, e))?;

        self.rebuild_index()?;

        Ok(format!("Memory '{}' deleted.", name))
    }

    fn list(&self) -> Result<Vec<String>, String> {
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

    fn search(&self, query: &str) -> Result<Vec<MemoryEntry>, String> {
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

    fn get_index_content(&self) -> String {
        let index_path = self.dir.join("MEMORY.md");
        fs::read_to_string(&index_path).unwrap_or_default()
    }
}

// ── Global memory store (backward-compatible wrapper) ──

static MEMORY_STORE: std::sync::LazyLock<std::sync::RwLock<Option<FileSystemMemoryStore>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(None));

pub fn init_memory_store(dir: &str) {
    let store = FileSystemMemoryStore::new(dir);
    let mut guard = MEMORY_STORE.write().unwrap();
    *guard = Some(store);
}

pub fn with_memory_store<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&dyn MemoryStorage) -> R,
{
    let guard = MEMORY_STORE.read().unwrap();
    guard.as_ref().map(|store| f(store as &dyn MemoryStorage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> (FileSystemMemoryStore, PathBuf) {
        let dir = std::env::temp_dir().join(format!("hmos_mem_{}", rand_id()));
        let _ = fs::remove_dir_all(&dir);
        let store = FileSystemMemoryStore::new(dir.to_str().unwrap());
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
            .save("test-mem", "A test memory", MemoryType::Knowledge, "Body content.")
            .unwrap();

        let mem = store.load("test-mem").unwrap();
        assert_eq!(mem.frontmatter.name, "test-mem");
        assert_eq!(mem.body, "Body content.");
        assert!(mem.tags.is_empty());
        assert!((mem.importance - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_list_and_delete() {
        let (store, _dir) = setup();
        store
            .save("m1", "Memory 1", MemoryType::Preference, "body 1")
            .unwrap();
        store
            .save("m2", "Memory 2", MemoryType::Knowledge, "body 2")
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
            .save("mem-a", "Memory A", MemoryType::Knowledge, "Rust coding patterns.")
            .unwrap();
        store
            .save("mem-b", "Memory B", MemoryType::Knowledge, "HarmonyOS platform tips.")
            .unwrap();

        let results = store.search("Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].frontmatter.name, "mem-a");
    }

    #[test]
    fn test_index_content() {
        let (store, _dir) = setup();
        store
            .save("idx-test", "Index test", MemoryType::Insight, "content")
            .unwrap();

        let idx = store.get_index_content();
        assert!(idx.contains("idx-test"));
        assert!(idx.contains("Index test"));
    }

    #[test]
    fn test_seven_memory_types_roundtrip() {
        let types = vec![
            MemoryType::Session,
            MemoryType::Conversation,
            MemoryType::Knowledge,
            MemoryType::Preference,
            MemoryType::Task,
            MemoryType::Error,
            MemoryType::Insight,
        ];
        for mt in &types {
            let s = mt.as_str();
            let resolved = MemoryType::resolve(s);
            assert_eq!(*mt, resolved);
        }
    }

    #[test]
    fn test_legacy_type_mapping() {
        assert_eq!(MemoryType::resolve("user"), MemoryType::Preference);
        assert_eq!(MemoryType::resolve("feedback"), MemoryType::Insight);
        assert_eq!(MemoryType::resolve("project"), MemoryType::Knowledge);
        assert_eq!(MemoryType::resolve("reference"), MemoryType::Knowledge);
    }

    #[test]
    fn test_importance_clamping() {
        let (store, _dir) = setup();
        store
            .save("test-imp", "Importance test", MemoryType::Knowledge, "body")
            .unwrap();

        let mem = store.load("test-imp").unwrap();
        assert!((0.0..=1.0).contains(&mem.importance));
    }

    #[test]
    fn test_memory_storage_trait_works() {
        let (store, _dir) = setup();
        // Test via trait object
        let s: &dyn MemoryStorage = &store;
        s.save("trait-test", "Trait test", MemoryType::Task, "body")
            .unwrap();
        let mem = s.load("trait-test").unwrap();
        assert_eq!(mem.frontmatter.name, "trait-test");
        assert_eq!(mem.frontmatter.memory_type, MemoryType::Task);
    }

    /// Integration test: simulate the actual tool handler call path
    /// (init_memory_store → with_memory_store → save/search/list).
    /// This catches the bug where `init_memory_store` is never called before tools are used.
    #[test]
    fn test_global_init_to_handler_path() {
        let dir = std::env::temp_dir().join(format!("hmos_mem_int_{}", rand_id()));
        let _ = fs::remove_dir_all(&dir);

        // Step 1: init global store (simulates json_router "init_session")
        // The json_router calls init_memory_store(&format!("{}/.memory", files_dir)),
        // so the argument already includes the ".memory" segment
        let memory_dir = dir.join(".memory");
        init_memory_store(memory_dir.to_str().unwrap());

        // Step 2: call save via with_memory_store (simulates memory_save_handler)
        let result = with_memory_store(|store| {
            store.save("integration-test", "Int test desc", MemoryType::Preference, "Test body")
        })
        .expect("memory store should be initialized")
        .expect("save should succeed");
        assert!(result.contains("saved"));

        // Step 3: call search via with_memory_store (simulates memory_search_handler)
        let results = with_memory_store(|store| store.search("Test body"))
            .expect("memory store should be initialized")
            .expect("search should succeed");
        assert!(!results.is_empty());
        assert_eq!(results[0].frontmatter.name, "integration-test");
        assert_eq!(results[0].frontmatter.memory_type, MemoryType::Preference);

        // Step 4: call list via with_memory_store (simulates memory_list_handler)
        let names = with_memory_store(|store| store.list())
            .expect("memory store should be initialized")
            .expect("list should succeed");
        assert!(names.contains(&"integration-test".to_string()));

        // Verify on-disk artifacts at the expected path
        assert!(memory_dir.exists(), "memory dir should exist after save");
        assert!(memory_dir.join("MEMORY.md").exists(), "MEMORY.md index should exist");
        assert!(
            memory_dir.join("integration-test.md").exists(),
            "integration-test.md file should exist"
        );

        fs::remove_dir_all(&dir).ok();
    }
}
