use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

/// Result of a single file match.
#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub file_path: String,
    pub line_number: u64,
    pub line_text: String,
}

/// In-process ripgrep search engine.
///
/// Uses the same `ignore` + `grep-*` crates that power ripgrep, running
/// entirely in-process (no subprocess, sandbox-compliant).
pub struct InProcessSearcher;

/// Collect results into a Vec<SearchMatch>.
#[derive(Default)]
struct MatchCollector {
    matches: Vec<SearchMatch>,
    current_path: String,
}

impl Sink for MatchCollector {
    type Error = std::io::Error;

    fn matched(
        &mut self,
        _searcher: &Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        let line = String::from_utf8_lossy(mat.bytes()).into_owned();
        self.matches.push(SearchMatch {
            file_path: self.current_path.clone(),
            line_number: mat.line_number().unwrap_or(0),
            line_text: line,
        });
        Ok(true)
    }
}

impl InProcessSearcher {
    /// Search the given directory for lines matching `pattern`.
    ///
    /// Respects `.gitignore` rules automatically via the `ignore` crate.
    pub fn search(dir: &Path, pattern: &str) -> Result<Vec<SearchMatch>, String> {
        let matcher = RegexMatcher::new(pattern)
            .map_err(|e| format!("invalid regex: {}", e))?;

        let mut searcher = SearcherBuilder::new()
            .binary_detection(grep_searcher::BinaryDetection::quit(b'\x00'))
            .build();

        let matches = Mutex::new(Vec::new());
        let walker = WalkBuilder::new(dir)
            .standard_filters(true) // .gitignore, hidden, etc.
            .build();

        for result in walker {
            let entry = result.map_err(|e| format!("walk error: {}", e))?;
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                continue;
            }

            let path = entry.path();
            let mut collector = MatchCollector {
                current_path: path.display().to_string(),
                ..Default::default()
            };

            let _ = searcher.search_path(&matcher, path, &mut collector);
            matches.lock().unwrap().extend(collector.matches);
        }

        Ok(matches.into_inner().unwrap())
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_search_in_temp_dir() {
        let dir = std::env::temp_dir().join("__hmos_search_test__");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut f = fs::File::create(dir.join("test.txt")).unwrap();
        writeln!(f, "hello world").unwrap();
        writeln!(f, "goodbye world").unwrap();

        let results = InProcessSearcher::search(&dir, "hello").unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| m.line_text.contains("hello")));

        let _ = fs::remove_dir_all(&dir);
    }
}
