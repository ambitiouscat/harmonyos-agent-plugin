use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Sandboxed file tool that restricts operations to a workspace root.
pub struct FileTools {
    workspace_root: PathBuf,
}

impl FileTools {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            workspace_root: PathBuf::from(workspace_root),
        }
    }

    /// Resolve a user-supplied path relative to the workspace root,
    /// rejecting any attempt to escape the sandbox via `..` traversal.
    fn resolve(&self, relative_path: &str) -> Result<PathBuf, String> {
        let clean = relative_path
            .trim_start_matches('/')
            .trim_start_matches('\\');
        let candidate = self.workspace_root.join(clean);
        let canonical = candidate
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;
        let root_canonical = self
            .workspace_root
            .canonicalize()
            .map_err(|e| format!("Workspace error: {}", e))?;
        if !canonical.starts_with(&root_canonical) {
            return Err("Path escapes workspace sandbox".into());
        }
        Ok(canonical)
    }

    pub fn read_file(&self, relative_path: &str) -> FileResult {
        match self.resolve(relative_path) {
            Ok(path) => {
                if !path.is_file() {
                    return FileResult {
                        success: false,
                        content: None,
                        error: Some("Not a file".into()),
                    };
                }
                match fs::read_to_string(&path) {
                    Ok(content) => FileResult {
                        success: true,
                        content: Some(content),
                        error: None,
                    },
                    Err(e) => FileResult {
                        success: false,
                        content: None,
                        error: Some(format!("Read failed: {}", e)),
                    },
                }
            }
            Err(e) => FileResult {
                success: false,
                content: None,
                error: Some(e),
            },
        }
    }

    pub fn write_file(&self, relative_path: &str, content: &str) -> FileResult {
        // For write, resolve parent dir to allow creating new files
        let clean = relative_path
            .trim_start_matches('/')
            .trim_start_matches('\\');
        let candidate = self.workspace_root.join(clean);

        // Ensure parent is within sandbox
        if let Some(parent) = candidate.parent() {
            let _ = fs::create_dir_all(parent);
            match parent.canonicalize() {
                Ok(ref p) => {
                    let root = self
                        .workspace_root
                        .canonicalize()
                        .unwrap_or_else(|_| self.workspace_root.clone());
                    if !p.starts_with(&root) {
                        return FileResult {
                            success: false,
                            content: None,
                            error: Some("Path escapes workspace sandbox".into()),
                        };
                    }
                }
                Err(e) => {
                    return FileResult {
                        success: false,
                        content: None,
                        error: Some(format!("Invalid parent path: {}", e)),
                    }
                }
            }
        }

        match fs::write(&candidate, content) {
            Ok(()) => FileResult {
                success: true,
                content: None,
                error: None,
            },
            Err(e) => FileResult {
                success: false,
                content: None,
                error: Some(format!("Write failed: {}", e)),
            },
        }
    }

    pub fn list_dir(&self, relative_path: &str) -> FileResult {
        match self.resolve(relative_path) {
            Ok(path) => {
                if !path.is_dir() {
                    return FileResult {
                        success: false,
                        content: None,
                        error: Some("Not a directory".into()),
                    };
                }
                let mut entries: Vec<String> = vec![];
                if let Ok(read_dir) = fs::read_dir(&path) {
                    for entry in read_dir.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let file_type = entry.file_type().map(|ft| {
                            if ft.is_dir() {
                                "/"
                            } else {
                                ""
                            }
                        }).unwrap_or("");
                        entries.push(format!("{}{}", name, file_type));
                    }
                }
                entries.sort();
                FileResult {
                    success: true,
                    content: Some(entries.join("\n")),
                    error: None,
                }
            }
            Err(e) => FileResult {
                success: false,
                content: None,
                error: Some(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_read_write_sandbox() {
        let dir = env::temp_dir().join("hmos_file_tools_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let tools = FileTools::new(dir.to_str().unwrap());

        // Write
        let r = tools.write_file("test.txt", "hello world");
        assert!(r.success);

        // Read
        let r = tools.read_file("test.txt");
        assert!(r.success);
        assert_eq!(r.content.unwrap(), "hello world");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sandbox_escape_blocked() {
        let dir = env::temp_dir().join("hmos_file_sandbox_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let tools = FileTools::new(dir.to_str().unwrap());
        let r = tools.read_file("../../etc/passwd");
        assert!(!r.success, "sandbox escape should be blocked");
        let err = r.error.unwrap();
        assert!(
            err.contains("sandbox") || err.contains("Invalid") || err.contains("escape"),
            "expected sandbox/escape error, got: {}",
            err
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_dir() {
        let dir = env::temp_dir().join("hmos_file_list_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.txt"), "b").unwrap();

        let tools = FileTools::new(dir.to_str().unwrap());
        let r = tools.list_dir(".");
        assert!(r.success);
        let content = r.content.unwrap();
        assert!(content.contains("a.txt"));
        assert!(content.contains("b.txt"));

        let _ = fs::remove_dir_all(&dir);
    }
}
