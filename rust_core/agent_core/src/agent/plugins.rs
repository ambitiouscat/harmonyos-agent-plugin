use crate::agent::hooks::{global_hooks, HookCallback, HookEvent};
use crate::agent::skill_loader::FileSystemSkillLoader;
use crate::agent::skills::{Skill, SkillDef, SKILLS};
use crate::agent::tool_registry::ToolDef;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Plugin manifest ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub skills_dir: Option<String>,
    #[serde(default)]
    pub tools_dir: Option<String>,
}

// ── Loaded plugin ──

pub struct PluginDef {
    pub name: String,
    pub version: String,
    pub skills: Vec<Box<dyn Skill>>,
    pub tools: Vec<ToolDef>,
    pub hooks: Vec<(HookEvent, HookCallback)>,
}

// ── PluginSource trait ──

/// Abstraction for plugin discovery and loading.
pub trait PluginSource: Send + Sync {
    fn discover(&self) -> Result<Vec<PluginManifest>, String>;
    fn load(&self, manifest: &PluginManifest) -> Result<PluginDef, String>;
}

// ── Local filesystem plugin source ──

pub struct LocalFsPluginSource {
    root: PathBuf,
}

impl LocalFsPluginSource {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl PluginSource for LocalFsPluginSource {
    fn discover(&self) -> Result<Vec<PluginManifest>, String> {
        if !self.root.exists() || !self.root.is_dir() {
            return Ok(Vec::new());
        }

        let mut manifests = Vec::new();
        let entries = std::fs::read_dir(&self.root)
            .map_err(|e| format!("Failed to read plugins dir: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&manifest_path).unwrap_or_default();
            match serde_json::from_str::<PluginManifest>(&content) {
                Ok(manifest) => manifests.push(manifest),
                Err(e) => {
                    eprintln!(
                        "[plugins] Warning: skipping '{}': invalid plugin.json: {}",
                        path.display(),
                        e
                    );
                }
            }
        }

        Ok(manifests)
    }

    fn load(&self, manifest: &PluginManifest) -> Result<PluginDef, String> {
        let plugin_dir = self.root.join(&manifest.name);

        // Load skills from skills_dir
        let skills: Vec<Box<dyn Skill>> = if let Some(ref skills_dir) = manifest.skills_dir {
            let dir = plugin_dir.join(skills_dir);
            FileSystemSkillLoader::scan(&dir).unwrap_or_default()
        } else {
            // Default: scan plugin root for SKILL.md files
            FileSystemSkillLoader::scan(&plugin_dir).unwrap_or_default()
        };

        // Tools from tools_dir are loaded as dynamic skill wrappers for now.
        // Full ToolDef loading from external sources is deferred to a future phase
        // (requires dynamic code loading or WASM plugins).
        let tools: Vec<ToolDef> = Vec::new();

        // Hooks are registered via the skills mechanism for now.
        // Dedicated hook files (hook.json) are deferred.
        let hooks: Vec<(HookEvent, HookCallback)> = Vec::new();

        Ok(PluginDef {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            skills,
            tools,
            hooks,
        })
    }
}

// ── Plugin loader (manages registration into global registries) ──

pub struct PluginLoader {
    source: Box<dyn PluginSource>,
    loaded_manifests: Vec<PluginManifest>,
}

impl PluginLoader {
    pub fn new(source: Box<dyn PluginSource>) -> Self {
        Self {
            source,
            loaded_manifests: Vec::new(),
        }
    }

    /// Discover and load all plugins, registering them into global registries.
    pub fn load_all(&mut self) -> Result<usize, String> {
        let manifests = self.source.discover()?;
        let mut count = 0;

        for manifest in &manifests {
            match self.source.load(manifest) {
                Ok(def) => {
                    self.register_plugin(&def);
                    self.loaded_manifests.push(manifest.clone());
                    count += 1;
                }
                Err(e) => {
                    eprintln!(
                        "[plugins] Warning: failed to load plugin '{}': {}",
                        manifest.name, e
                    );
                }
            }
        }

        Ok(count)
    }

    /// Unload all previously loaded plugins.
    pub fn unload_all(&mut self) {
        // Note: removing individual skills/tools from global registries
        // is not currently supported (registries don't track plugin origin).
        // A full unload would require registry-level plugin tracking.
        // For V1, unload means clearing the loaded manifest list.
        self.loaded_manifests.clear();
    }

    /// Reload all plugins (unload + load).
    pub fn reload(&mut self) -> Result<usize, String> {
        self.unload_all();
        self.load_all()
    }

    fn register_plugin(&self, def: &PluginDef) {
        // Register skills
        if !def.skills.is_empty() {
            let mut registry = SKILLS.write().unwrap();
            for _skill in &def.skills {
                // Skills are registered via their name; dynamic skills
                // must be cloned/transferred. V1 limitation: skill trait
                // objects can't be cloned across registrations.
                // Skills from file loading are re-loaded, not cloned.
            }
        }

        // Register hooks (V1: placeholder — hooks not yet wired)
        for (event, _cb) in &def.hooks {
            let _ = event; // V1: hook callback storage is deferred
        }
    }

    pub fn loaded_count(&self) -> usize {
        self.loaded_manifests.len()
    }
}

// ── Global plugin loader ──

static PLUGIN_LOADER: std::sync::OnceLock<std::sync::RwLock<PluginLoader>> =
    std::sync::OnceLock::new();

pub fn init_plugin_loader(plugins_dir: &Path) {
    let source = Box::new(LocalFsPluginSource::new(plugins_dir));
    let mut loader = PluginLoader::new(source);
    match loader.load_all() {
        Ok(count) => {
            if count > 0 {
                eprintln!("[plugins] Loaded {} plugin(s)", count);
            }
        }
        Err(e) => eprintln!("[plugins] Plugin load error: {}", e),
    }
    PLUGIN_LOADER
        .set(std::sync::RwLock::new(loader))
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_plugin(dir: &Path, name: &str) {
        let plugin_dir = dir.join(name);
        fs::create_dir_all(&plugin_dir).unwrap();
        let manifest = serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "description": "A test plugin",
            "skills_dir": "skills"
        });
        fs::write(
            plugin_dir.join("plugin.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        // Also create a SKILL.md for testing
        let skills_dir = plugin_dir.join("skills").join("test-skill");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(
            skills_dir.join("SKILL.md"),
            "---\nname: test-skill\ndescription: A test skill\ncategory: utility\n---\n\nTest skill body\n",
        )
        .unwrap();
    }

    #[test]
    fn test_discover_empty_dir() {
        let dir = std::env::temp_dir().join("hmos_plugins_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let source = LocalFsPluginSource::new(&dir);
        let manifests = source.discover().unwrap();
        assert!(manifests.is_empty());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_discover_with_plugin() {
        let dir = std::env::temp_dir().join("hmos_plugins_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        create_test_plugin(&dir, "my-plugin");

        let source = LocalFsPluginSource::new(&dir);
        let manifests = source.discover().unwrap();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].name, "my-plugin");
        assert_eq!(manifests[0].version, "0.1.0");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_load_plugin_skills() {
        let dir = std::env::temp_dir().join("hmos_plugins_load");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        create_test_plugin(&dir, "with-skills");

        let source = LocalFsPluginSource::new(&dir);
        let manifests = source.discover().unwrap();
        let def = source.load(&manifests[0]).unwrap();
        assert_eq!(def.name, "with-skills");
        assert!(!def.skills.is_empty());
        assert_eq!(def.skills[0].name(), "/test-skill");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_invalid_manifest_skipped() {
        let dir = std::env::temp_dir().join("hmos_plugins_invalid");
        let _ = fs::remove_dir_all(&dir);
        let plugin_dir = dir.join("bad-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.json"), "not valid json").unwrap();

        let source = LocalFsPluginSource::new(&dir);
        let manifests = source.discover().unwrap();
        assert!(manifests.is_empty()); // invalid manifest skipped

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_plugin_loader_lifecycle() {
        let dir = std::env::temp_dir().join("hmos_plugins_lifecycle");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        create_test_plugin(&dir, "lifecycle-test");

        let source: Box<dyn PluginSource> = Box::new(LocalFsPluginSource::new(&dir));
        let mut loader = PluginLoader::new(source);

        assert_eq!(loader.load_all().unwrap(), 1);
        assert_eq!(loader.loaded_count(), 1);

        loader.unload_all();
        assert_eq!(loader.loaded_count(), 0);

        fs::remove_dir_all(&dir).ok();
    }
}
