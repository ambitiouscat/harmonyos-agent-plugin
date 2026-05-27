---
generated_at: 2026-05-27
chapters:
  - Entry Points & Initialization
  - Cross-Boundary Communication
  - LLM Chat & SSE Streaming
  - ReAct Agent Loop
  - Session Management
  - Skills & Commands
  - RAG & Search
  - Tool Execution
  - State Management & UI
  - Technical Debt & Known Issues
---

# Core Content

## 1. Entry Points & Initialization

### HarmonyOS App Startup
1. `EntryAbility.ets` → `onWindowStageCreate()` → loads `pages/Index`
2. `Index.ets` (build-only, `@Entry`) → instantiates `ChatView`, pre-loads `PhraseLoader`
3. `ChatView.aboutToAppear()` → initializes `RustAgentBridge`
4. `RustAgentBridge.init(apiKey, baseUrl, model, filesDir)` → calls `libnative_bridge.so` → `initAgentWithIo()`
5. C++ `InitAgentWithIo()` → `rust_agent_init(callbacks)` → stores IO callbacks in Rust
6. Rust stores `post_fn`, `stream_post_fn`, `free_str_fn` for host IO

### Rust Core Initialization
- `rust_agent_init(callbacks: SystemCallbacks)` — stores callbacks, returns boolean
- `rust_agent_init_session(files_dir)` — sets working directory for session JSON files
- Config state stored in thread-local or global `OnceLock`/`Mutex`

### Platform Variants
- **HarmonyOS**: Host IO through callbacks (HTTP goes through `SystemIoImpl.ets`)
- **WASM**: JS functions bridged via `OnceLock<js_sys::Function>`
- **Node.js**: Direct Rust IO (ureq + std::fs), no host callbacks needed

## 2. Cross-Boundary Communication

### Layer Stack
```
[ArkTS ChatView]
    ↓ NAPI direct call
[C++ native_bridge.cpp]
    ↓ extern "C" FFI
[Rust ffi.rs → json_router.rs]
    ↓ internal dispatch
[Rust agent/tools modules]
```

### Call Flow (ArkTS → Rust)
1. `RustAgentBridge.call(action, jsonArgs)` → NAPI `agentCall(action, jsonArgs)`
2. C++ `AgentCall()` → `rust_agent_call(action, jsonArgs)` → `*mut c_char`
3. C++ wraps result in NAPI string, returns to ArkTS
4. ArkTS parses JSON response

### Callback Flow (Rust → ArkTS)
1. Rust calls `stream_post_fn(chunk_data, event_type)` for SSE chunks
2. Or `post_fn(response_json)` for final result
3. C++ `napi_threadsafe_function` marshals call to ArkTS main thread
4. `RustAgentBridge` receives callback, resolves Promise or triggers SSE parsing

### Action Router (~20 actions)
- `ping` — health check, returns `{"status":"pong"}`
- `chat` — main LLM chat with streaming
- `configure` — update API key, base URL, model
- `abort` / `reset_abort` — cancel current operation
- `init_session` — initialize session file directory
- `create_session` / `load_session` / `list_sessions` / `delete_session` / `save_session`
- `get_registered_skills` — list available /commands
- `vfs_write_file` — write file via sandbox
- `test_stream` — simulate streaming for testing

## 3. LLM Chat & SSE Streaming

### Chat Flow (Rust side)
1. `json_router` dispatches `chat` action to `agent/chat.rs`
2. Builds messages array from request (system prompt + conversation history)
3. Opens HTTP POST to LLM provider with `stream: true`
4. Uses `ureq` with rustls for HTTPS
5. Reads SSE response line by line
6. Each `data: {...}` line parsed as JSON
7. Delta text extracted and sent via `stream_post_fn(chunk, "text")`
8. Tool calls sent via `stream_post_fn(chunk, "tool_call")`
9. Final message sent via `post_fn(complete_response)`

### SSE Processing (ArkTS side)
1. `SseStreamController` receives raw chunk data
2. Parses `data: ...` lines, accumulates partial JSON
3. Extracts text delta from `choices[0].delta.content`
4. Uses interleaved timers (~20ms delay) between chunks
5. Timer yields control to ArkUI render loop → prevents UI freeze
6. Supports cancellation via `AbortController`

### Reconnection
- `ReconnectConfig`: max_retries=3, base_delay_ms=500, max_delay_ms=5000
- Exponential backoff on connection failure
- `SseMerger` accumulates partial SSE JSON across reconnections

### Frame Throttling
- `FrameThrottler<T>` batches items within ~16ms windows
- Prevents excessive UI re-renders during high-frequency streaming

## 4. ReAct Agent Loop

### Loop Engine (`agent/loop_engine.rs`)

```
Loop State:
  iteration: 0..30
  message_history: Vec<Message>
  tool_results: Vec<ContentPart>
  fingerprint_window: [SHA-256; 5]

Each Iteration:
  1. ContextManager.build(messages, tools) → formatted prompt
  2. LLM call via chat.rs
  3. Parse response → ContentPart[]
  4. For each ContentPart:
     - Text: append to output
     - ToolCall: execute tool, append ToolResult
  5. Loop detection: SHA-256(tool_name + args) vs window
  6. Check: finished? loop? max iterations? user abort?
```

### Loop Detection
- SHA-256 hash of `{tool_name}{serialized_args}`
- Sliding window of last 5 hashes
- Exact match → loop detected → stop with `LoopDetected`
- Different args to same tool = different hash = allowed

### Termination Conditions
- `Completed` — LLM returns finish_reason="stop"
- `StoppedByUser` — AtomicBool abort flag set
- `LoopDetected` — SHA-256 collision in sliding window
- `MaxIterationsReached` — 30 iterations exceeded

### Context Management (`agent/context.rs`)
- Estimates token count (chars/3.5 heuristic)
- 80% threshold triggers compression
- Keeps system prompt intact
- Trims oldest messages first (preserves last 4)
- Removes tool output from trimmed messages

## 5. Session Management

### Session Model
```rust
Session {
    id: String,          // UUID v4 style
    title: String,
    messages: Vec<Message>,
    created_at: String,  // ISO 8601
    updated_at: String,  // ISO 8601
    model: Option<String>,
}

SessionMeta {
    id: String,
    title: String,
    message_count: usize,
    created_at: String,
    updated_at: String,
}
```

### Storage
- JSON files in `filesDir/sessions/`
- One file per session: `{session_id}.json`
- CRUD operations: create, load, list, save, delete
- `SessionManager` handles file IO with error handling

### ArkTS Integration
- `ChatView` manages session lifecycle
- `ConversationList` displays session drawer (80% width)
- Session CRUD via `RustAgentBridge.call('create_session', ...)` etc.
- New sessions auto-created on first message
- Auto-save after each message exchange

## 6. Skills & Commands

### Built-in Skills (5 commands)
| Command | Description | Parameters |
|---------|-------------|------------|
| `/search` | Search files with pattern | `pattern` (required), `dir_path` (optional) |
| `/scan` | Build RAG index of directory | `dir_path` (required) |
| `/goal` | Set or view agent goal | `goal_text` (optional) |
| `/file` | Read/write/list files | `operation` (read/write/list), `path`, `content` |
| `/session` | Session operations | `operation` (new/list/switch/delete/save), `id` |

### Skill Definition
```rust
SkillDef {
    name: String,          // e.g., "search"
    description: String,   // "Search files using patterns"
    params: Vec<SkillParam>,
    category: String,      // "tools", "session", etc.
}
```

### Command Palette (ArkTS)
- `InputBar` detects `/` prefix → triggers `CommandPalette`
- `CommandPalette` fetches skills via `get_registered_skills`
- Prefix matching and fuzzy matching
- Arrow key selection (selectedIdx)
- Max 8 results displayed
- Category labels on each result

### Phrase System
- `agent_thinking_phrases.json` — categorized thinking phrases
- Categories: default, cartoon, academic, poetic, etc.
- `PhraseLoader` caches phrases, provides random selection
- `ThinkingPanel` displays rotating phrases with spinner
- Configurable via SettingsPage phrase style selector

## 7. RAG & Search

### BM25 RAG (`agent/rag.rs`)
- `RagIndex` — in-memory BM25 heuristic index
- Indexing: parallel multi-threaded file chunk processing
- Supported file types: md, txt, rs, ets, ts, cpp, h, json5
- Tokenizes content, builds inverted index
- Query: BM25 scoring with IDF weighting
- Filters short tokens (min length threshold)

### In-Process Search (`agent/search.rs`)
- Uses ripgrep crates: `ignore` + `grep-regex` + `grep-searcher`
- Respects `.gitignore` rules via `ignore` crate
- Returns `SearchMatch { file_path, line_number, line_text }`
- Exposed via `rust_agent_search(dir_path, pattern)` FFI
- `rust_agent_scan_dir(dir_path)` builds RAG index

### Integration
- `/search` skill → `search.rs` → results to LLM context
- `/scan` skill → `rag.rs` → index built, available for retrieval
- Automatic: ReAct loop can trigger RAG retrieval for context augmentation

## 8. Tool Execution

### File Tools (`tools/file.rs`)
- `read(path)` — Read file content, canonicalize path first
- `write(path, content)` — Write file, validate path traversal
- `list(dir)` — List directory contents
- **Safety**: `canonicalize()` + prefix check blocks path traversal attacks
- All operations go through host IO callbacks (HarmonyOS) or direct fs (Node)

### MCP Bridge (`tools/mcp.rs`)
- MCP client via stdio JSON-RPC
- Manages subprocess lifecycle (spawn/kill)
- `McpServer` config: command, args, env
- `McpTool` discovery: list_tools request
- Tool calls: call_tool request with arguments
- Multiple MCP servers supported

### SubAgent Runner (`tools/subagent.rs`)
- Spawns sub-agents in separate threads
- `SubAgentConfig`: prompt, tools, model
- `SubAgentProgress` callback for status updates
- Synchronous and async modes
- Progress reported via callback

### Web Tools (`tools/web.rs`)
- Host-based HTTP (goes through ArkTS `SystemIoImpl.ets`)
- Web search: DDG HTML parsing, result extraction
- Web fetch: URL → stripped HTML/text content
- Basic HTML tag stripping (keeps text content)
- `WebSearchResult { title, url, snippet }`
- `WebFetchResult { url, content, content_type }`

## 9. State Management & UI

### ArkTS Data Model
```
ArkMessage (@ObservedV2)
├── id: string
├── role: 'system' | 'user' | 'assistant' | 'tool'
├── model: string | null
├── createdAt: string
└── parts: ArkContentPart[] (@Trace)

ArkContentPart (@ObservedV2)
├── type: 'text' | 'reasoning' | 'tool_call' | 'tool_result'
├── text: string (@Trace)
├── collapsed: boolean (@Trace)
├── toolId: string | null
├── name: string | null
└── isError: boolean (@Trace)
```

### Streaming UI Updates
1. SSE chunk arrives → `SseStreamController` parses delta
2. `ChatView.applyDelta(messageId, delta)` → mutates `@Trace` fields
3. ArkUI detects `@Trace` change → re-renders only affected nodes
4. For new messages: create `ArkMessage` + `ArkContentPart`, push to array

### Component Tree
```
Index (@Entry)
└── ChatView
    ├── TopBar (hamburger + title + settings gear)
    ├── ConversationList (80% drawer, conditional)
    ├── List
    │   └── ForEach → MessageBubble
    │       ├── MarkdownView (text parts)
    │       ├── ThinkingPanel (reasoning parts)
    │       └── ToolCard (tool_call/tool_result parts)
    ├── StatusBar (spinner during streaming)
    └── InputBar
        └── CommandPalette (conditional on /)
```

### Settings Persistence
- HarmonyOS `preferences` API (key-value store)
- Stored: apiKey, baseUrl, model, provider, phraseStyle
- Loaded on `SettingsPage.aboutToAppear()`
- Applied to `RustAgentBridge.configure()` immediately

## 10. Technical Debt & Known Issues

### Current State (2026-05-27)
- Active development on `main` branch
- 5 staged files + 9 untracked files (new conversation UI + command palette)
- Recent commits: Phase 5 features (thinking phrases, spinner engine, lifecycle fixes)

### Architecture Concerns
1. **No error boundary pattern** in ArkTS — NAPI call failures may crash UI
2. **Session file locking** — concurrent access to session JSON files not handled
3. **RAG index in-memory only** — lost on process restart, no persistence
4. **MCP subprocess lifecycle** — no health check or auto-restart for MCP servers
5. **Token estimation heuristic** (chars/3.5) — not accurate for CJK text
6. **No telemetry/logging aggregation** — debugging requires device connection

### Known Limitations
- OHOS musl missing symbols require `musl_shim.c` compatibility stubs
- Cross-compilation requires WSL2 (not native Windows)
- WASM build cannot use ripgrep (file system access limited)
- Node.js build uses direct fs (no sandbox parity with HarmonyOS)
- Command palette limits to 8 results (no scrolling)
- No i18n/l10n support (Chinese UI hardcoded in some places)

### Test Coverage Gaps
- ArkTS: No unit tests (only manual testing)
- C++ NAPI: No unit tests
- Rust: Good coverage (~35 unit + 7 integration) but no property-based tests
- Cross-boundary: No end-to-end integration tests for full ArkTS→Rust→LLM flow

### Upcoming Work (from git status)
- New `ui/conversation/` and `ui/input/CommandPalette.ets` being added
- Rust side: new `agent/session.rs`, `agent/skills.rs`, `tools/` directory
- Refactoring of `ffi.rs`, `json_router.rs`, `types/message.rs`

