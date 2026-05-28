use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Sandbox path resolver shared with file tools.
fn resolve_sandbox_path(relative_path: &str, sandbox_root: &str) -> Result<PathBuf, String> {
    let clean = relative_path.trim_start_matches('/').trim_start_matches('\\');
    let candidate = PathBuf::from(sandbox_root).join(clean);
    let canonical = candidate
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;
    let root_canonical = PathBuf::from(sandbox_root)
        .canonicalize()
        .map_err(|e| format!("Workspace error: {}", e))?;
    if !canonical.starts_with(&root_canonical) {
        return Err("Path escapes workspace sandbox".into());
    }
    Ok(canonical)
}

/// Edit a file by replacing an exact string match.
///
/// Returns an error if old_string is not unique in the file (0 or >1 matches).
pub fn edit_handler(args: Value, sandbox_root: &str) -> Result<String, String> {
    let file_path = args["file_path"]
        .as_str()
        .ok_or_else(|| "Missing 'file_path' parameter".to_string())?;
    let old_string = args["old_string"]
        .as_str()
        .ok_or_else(|| "Missing 'old_string' parameter".to_string())?;
    let new_string = args["new_string"]
        .as_str()
        .ok_or_else(|| "Missing 'new_string' parameter".to_string())?;

    let path = resolve_sandbox_path(file_path, sandbox_root)?;

    if !path.is_file() {
        return Err(format!("Not a file: {}", file_path));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

    let matches: Vec<_> = content.match_indices(old_string).collect();

    if matches.is_empty() {
        return Err(format!(
            "old_string not found in {}. The string to replace must match exactly, including whitespace.",
            file_path
        ));
    }

    if matches.len() > 1 {
        return Err(format!(
            "old_string found {} times in {}. Provide a larger string with more surrounding context to make it unique.",
            matches.len(),
            file_path
        ));
    }

    let (pos, _) = matches[0];
    let mut new_content = String::with_capacity(content.len());
    new_content.push_str(&content[..pos]);
    new_content.push_str(new_string);
    new_content.push_str(&content[pos + old_string.len()..]);

    fs::write(&path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

    Ok(format!("Successfully edited {}", file_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir() -> (String, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap().to_string();
        (root, dir)
    }

    #[test]
    fn test_edit_success() {
        let (root, _dir) = setup_test_dir();
        let file_path = std::path::PathBuf::from(&root).join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let result = edit_handler(
            serde_json::json!({
                "file_path": "test.txt",
                "old_string": "hello",
                "new_string": "goodbye"
            }),
            &root,
        );
        assert!(result.is_ok());

        let updated = fs::read_to_string(&file_path).unwrap();
        assert_eq!(updated, "goodbye world");
    }

    #[test]
    fn test_edit_not_found() {
        let (root, _dir) = setup_test_dir();
        fs::write(
            std::path::PathBuf::from(&root).join("test.txt"),
            "hello world",
        ).unwrap();

        let result = edit_handler(
            serde_json::json!({
                "file_path": "test.txt",
                "old_string": "nope",
                "new_string": "x"
            }),
            &root,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_edit_multiple_matches() {
        let (root, _dir) = setup_test_dir();
        fs::write(
            std::path::PathBuf::from(&root).join("test.txt"),
            "hello hello",
        ).unwrap();

        let result = edit_handler(
            serde_json::json!({
                "file_path": "test.txt",
                "old_string": "hello",
                "new_string": "x"
            }),
            &root,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("2 times"));
    }

    #[test]
    fn test_edit_sandbox_escape() {
        let (root, _dir) = setup_test_dir();

        let result = edit_handler(
            serde_json::json!({
                "file_path": "../../etc/passwd",
                "old_string": "x",
                "new_string": "y"
            }),
            &root,
        );
        assert!(result.is_err());
    }
}
