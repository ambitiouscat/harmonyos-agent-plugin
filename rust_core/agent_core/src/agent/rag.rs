use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

/// A single chunk of a document, ready for indexing.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub file_path: PathBuf,
    pub index: usize,
    pub text: String,
}

/// BM25-inspired keyword index (Level 0 RAG).
///
/// Phase 1 implements a simple TF-based inverted index. Full BM25 scoring,
/// embedding support, and async background scanning arrive in later phases.
#[derive(Debug, Default)]
pub struct RagIndex {
    /// word → (file → term frequency)
    inverted: HashMap<String, HashMap<PathBuf, usize>>,
    chunks: Vec<Chunk>,
}

impl RagIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a directory tree and index all `.md` and `.txt` files.
    pub fn scan_dir(&mut self, dir: &Path) -> Result<usize, String> {
        let entries = walk_files(dir)?;
        let chunks = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        // Simple multi-threaded chunking.
        for batch in entries.chunks(8) {
            let batch: Vec<_> = batch.to_vec();
            let chunks = Arc::clone(&chunks);
            handles.push(thread::spawn(move || {
                for path in batch {
                    if let Ok(chunk_list) = chunk_file(&path) {
                        chunks.lock().unwrap().extend(chunk_list);
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let chunks = Arc::try_unwrap(chunks).unwrap().into_inner().unwrap();

        // Build inverted index.
        for chunk in &chunks {
            for word in tokenize(&chunk.text) {
                let file_map = self.inverted.entry(word).or_default();
                *file_map.entry(chunk.file_path.clone()).or_insert(0) += 1;
            }
        }

        let count = chunks.len();
        self.chunks = chunks;
        Ok(count)
    }

    /// Keyword search — returns chunks whose text contains the query terms.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<&Chunk> {
        let query_terms: Vec<String> = tokenize(query);

        // Score each chunk by term frequency in the query.
        let mut scored: Vec<(&Chunk, usize)> = self
            .chunks
            .iter()
            .map(|chunk| {
                let score: usize = query_terms
                    .iter()
                    .filter(|t| chunk.text.contains(t.as_str()))
                    .count();
                (chunk, score)
            })
            .filter(|(_, s)| *s > 0)
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.truncate(top_k);
        scored.into_iter().map(|(c, _)| c).collect()
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

fn walk_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    walk_inner(dir, &mut files).map_err(|e| format!("walk error: {}", e))?;
    Ok(files)
}

fn walk_inner(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_inner(&path, files)?;
        } else {
            match path.extension().and_then(|e| e.to_str()) {
                Some("md" | "txt" | "rs" | "ets" | "ts" | "cpp" | "h" | "json5") => {
                    files.push(path);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn chunk_file(path: &Path) -> Result<Vec<Chunk>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content
        .split("\n\n")
        .enumerate()
        .filter(|(_, p)| !p.trim().is_empty())
        .map(|(i, p)| Chunk {
            file_path: path.to_path_buf(),
            index: i,
            text: p.trim().to_string(),
        })
        .collect())
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 2)
        .map(|s| s.to_lowercase())
        .collect()
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_scan_and_search() {
        let dir = std::env::temp_dir().join("__hmos_rag_test__");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut f = fs::File::create(dir.join("notes.md")).unwrap();
        writeln!(f, "Rust is a systems programming language.\n\nIt provides memory safety without garbage collection.").unwrap();

        let mut idx = RagIndex::new();
        let count = idx.scan_dir(&dir).unwrap();
        assert!(count > 0);
        assert_eq!(count, idx.chunk_count());

        let results = idx.search("Rust memory", 3);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_tokenize_filters_short_tokens() {
        let tokens = tokenize("a b c hello world ab");
        assert_eq!(tokens, vec!["hello", "world", "ab"]);
    }
}
