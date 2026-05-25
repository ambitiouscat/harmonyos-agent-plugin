## ADDED Requirements

### Requirement: In-process file search

The system SHALL provide file-content search using the `ignore` and `grep-*` Rust crates, running entirely in-process without subprocess invocation. Search SHALL respect `.gitignore` rules automatically.

#### Scenario: Single file match
- **WHEN** `InProcessSearcher::search(dir, "pattern")` is called with a directory containing matching files
- **THEN** a `Vec<SearchMatch>` is returned with file_path, line_number, and line_text for each match

#### Scenario: No matches
- **WHEN** the pattern does not match any file content
- **THEN** an empty Vec is returned (not an error)

#### Scenario: Invalid regex
- **WHEN** the pattern is invalid regex syntax
- **THEN** `Err(String)` is returned with the regex error message

#### Scenario: Permission denied
- **WHEN** the directory is not readable by the process
- **THEN** `Err(String)` is returned with the OS error description

### Requirement: C FFI export for search

The search capability SHALL be callable from C via `rust_agent_search(dir_path: *const c_char, pattern: *const c_char) -> *mut c_char`, returning a JSON array of matches. The caller MUST free the returned pointer via `rust_agent_free_str`.

#### Scenario: Search via C FFI
- **WHEN** ArkTS calls `nativeBridge.search("/sandbox", "error")`
- **THEN** a JSON string array of matches is returned
- **AND** the response is freed by `rust_agent_free_str`
