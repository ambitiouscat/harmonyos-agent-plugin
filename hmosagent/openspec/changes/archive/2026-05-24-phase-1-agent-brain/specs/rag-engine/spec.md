## ADDED Requirements

### Requirement: Directory scanning and chunking

The system SHALL scan a directory tree recursively, indexing files with extensions `.md`, `.txt`, `.rs`, `.ets`, `.ts`, `.cpp`, `.h`, `.json5`. Documents SHALL be chunked on paragraph boundaries (`\n\n`). Scanning SHALL use multiple threads for parallel processing.

#### Scenario: Scan directory with markdown files
- **WHEN** `RagIndex::scan_dir(dir)` is called on a directory containing `.md` files
- **THEN** chunks are created and indexed
- **AND** `scan_dir` returns `Ok(count)` where count > 0

#### Scenario: Empty directory
- **WHEN** the directory contains no indexable files
- **THEN** `Ok(0)` is returned (not an error)

### Requirement: BM25 keyword search

The system SHALL provide keyword search via `RagIndex::search(query, top_k)`, tokenizing the query on non-alphanumeric boundaries (minimum 2-character tokens) and scoring chunks by term-frequency intersection.

#### Scenario: Keyword match
- **WHEN** `search("Rust memory", 3)` is called on an index containing Rust-related documents
- **THEN** the top-ranked chunks contain the tokens "rust" and/or "memory"

#### Scenario: No match
- **WHEN** the query contains no terms present in the index
- **THEN** an empty Vec is returned

### Requirement: C FFI export for scan

The scan capability SHALL be callable from C via `rust_agent_scan_dir(dir_path: *const c_char) -> *mut c_char`, returning JSON `{"status":"ok","chunks_indexed":N}`. The caller MUST free the returned pointer via `rust_agent_free_str`.

#### Scenario: Scan via C FFI
- **WHEN** ArkTS calls `nativeBridge.scanDir("/sandbox")`
- **THEN** JSON with status and chunks_indexed is returned
