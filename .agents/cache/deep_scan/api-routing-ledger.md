---
generated_at: 2026-05-27
endpoint_count: 36
sections:
  - C FFI Surface (9 functions)
  - JSON Action Router (14 actions)
  - NAPI Bridge Functions (7 functions)
  - ArkTS Public API (9 exports)
  - Built-in Skills (5 commands)
  - Host IO Callbacks (4 callbacks)
  - Session CRUD API (5 operations)
  - MCP Bridge API (6 methods)
---

# API Routing Ledger

## A. C FFI Surface (`extern "C"` functions)

| Function | Parameters | Return | Description | Source |
|----------|-----------|--------|-------------|--------|
| `rust_agent_init` | `callbacks: SystemCallbacks` | `bool` | Initialize Rust core with host IO capabilities | `rust_core/.../ffi.rs:34` |
| `rust_agent_init_session` | `files_dir: *const c_char` | `bool` | Initialize session manager with sandbox directory | `rust_core/.../ffi.rs:43` |
| `rust_agent_call` | `action: *const c_char, json_args: *const c_char` | `*mut c_char` | Unified JSON action dispatcher. Caller must free result via `rust_agent_free_str` | `rust_core/.../ffi.rs:55` |
| `rust_agent_free_str` | `ptr: *mut c_char` | `void` | Free string returned by `rust_agent_call` | `rust_core/.../ffi.rs:76` |
| `rust_agent_search` | `dir_path: *const c_char, pattern: *const c_char` | `*mut c_char` | In-process ripgrep search (not wasm32) | `rust_core/.../ffi.rs:90` |
| `rust_agent_scan_dir` | `dir_path: *const c_char` | `*mut c_char` | Build BM25 RAG index (not wasm32) | `rust_core/.../ffi.rs:117` |
| `rust_agent_register_stream_cb` | `callback: extern "C" fn(*const c_char, u8)` | `void` | Register C callback for stream chunks | `rust_core/.../sandbox/validate.rs:12` |
| `test_network` | `(none)` | `bool` | TCP connect to 8.8.8.8:53 for sandbox health check | `rust_core/.../sandbox/validate.rs:22` |
| `test_file` | `dir: *const c_char` | `bool` | Write+delete temp file to verify sandbox writes | `rust_core/.../sandbox/validate.rs:31` |

## B. JSON Action Router (action → handler mapping)

All dispatched via `rust_agent_call(action, json_args)`. Response format: `AgentResponse { status, message?, error? }`.

| Action String | Handler | Input Format | Output | Source |
|---------------|---------|-------------|--------|--------|
| `"ping"` | inline | `{}` (ignored) | `{"status":"ok","message":"pong"}` | `json_router.rs:29` |
| `"init_session"` | inline | `{"files_dir":"<path>"}` | `{"status":"ok","message":"Session manager initialized"}` | `json_router.rs:34` |
| `"get_registered_skills"` | inline | `{}` | Serialized `Vec<SkillDef>` JSON | `json_router.rs:52` |
| `"load_session"` | inline | `{"session_id":"<id>"}` | Serialized `Session` JSON | `json_router.rs:61` |
| `"create_session"` | inline | `{"title":"<title>"}` | Serialized `Session` JSON | `json_router.rs:103` |
| `"list_sessions"` | inline | `{}` | Serialized `Vec<SessionMeta>` JSON | `json_router.rs:137` |
| `"delete_session"` | inline | `{"session_id":"<id>"}` | `{"status":"ok","message":"Session deleted"}` | `json_router.rs:159` |
| `"save_session"` | inline | Full `Session` object JSON | `{"status":"ok","message":"Session saved"}` | `json_router.rs:199` |
| `"configure"` | inline | Arbitrary JSON value (apiKey, baseUrl, model, etc.) | `{"status":"ok","message":"Configuration stored"}` | `json_router.rs:230` |
| `"abort"` | inline | `{}` | `{"status":"ok","message":"Generation aborted"}` | `json_router.rs:249` |
| `"reset_abort"` | inline | `{}` | `{"status":"ok","message":"Abort flag reset"}` | `json_router.rs:257` |
| `"vfs_write_file"` | inline (wasm32 only) | `AgentRequest::VfsWrite { path, content }` | Error on non-wasm32 | `json_router.rs:265` |
| `"chat"` | inline → background thread | `AgentRequest::ChatStream { messages }` | `{"status":"streaming"}` then stream chunks via callback | `json_router.rs:287` |
| `"test_stream"` | inline | `TestStream { chunks: u32, interval_ms: u64 }` | `{"status":"streaming"}` then simulated chunks | `json_router.rs:345` |
| `_` (unknown) | inline | Any | `{"status":"error","error":"Unknown action: <action>"}` | `json_router.rs:369` |

## C. NAPI Bridge Functions (ArkTS → C++ → Rust)

| ArkTS Method | NAPI C++ Func | FFI Rust Func | Parameters | Description |
|-------------|---------------|---------------|------------|-------------|
| `RustAgentBridge.init(onChunk)` | `InitAgent` | `rust_agent_register_stream_cb` + `rust_agent_init` | `(StreamCallback) => boolean` | Phase 0 init (stream-only callbacks) |
| `RustAgentBridge.initWithIo(onChunk, onIo)` | `InitAgentWithIo` | `rust_agent_register_stream_cb` + `rust_agent_init` | `(StreamCallback, IoCallback) => boolean` | Phase 1 init (full IO injection) |
| `RustAgentBridge.call(action, jsonArgs)` | `AgentCall` | `rust_agent_call` + `rust_agent_free_str` | `(string, string) => string` | Dispatch any JSON action |
| `RustAgentBridge.testNetwork()` | `TestNetwork` | `test_network` | `() => boolean` | Sandbox network check |
| `RustAgentBridge.testFile(dir)` | `TestFile` | `test_file` | `(string) => boolean` | Sandbox file write check |
| `RustAgentBridge.search(dirPath, pattern)` | `Search` | `rust_agent_search` + `rust_agent_free_str` | `(string, string) => string` | In-process ripgrep search |
| `RustAgentBridge.scanDir(dirPath)` | `ScanDir` | `rust_agent_scan_dir` + `rust_agent_free_str` | `(string) => string` | Build RAG index |

## D. ArkTS Public API (HAR exports from `Index.ets`)

| Export | Type | Source File | Description |
|--------|------|-------------|-------------|
| `RustAgentBridge` | class (singleton) | `RustAgentBridge.ets` | Primary bridge to Rust core |
| `ChatView` | @ComponentV2 | `ui/chat/ChatView.ets` | Main chat UI scaffold |
| `SettingsPage` | @ComponentV2 | `pages/SettingsPage.ets` | Configuration page |
| `ArkMessage` | @ObservedV2 class | `ui/models/ArkMessage.ets` | Message data model |
| `ArkContentPart` | @ObservedV2 class | `ui/models/ArkContentPart.ets` | Content part model |
| `getPhraseLoader` | function | `ui/common/PhraseLoader.ets` | Phrase loader accessor |
| `CMD_WEB2CPP` | const `"agent:web2cpp"` | `Index.ets:8` | IPC command constant |
| `EVENT_CPP2WEB` | const `"agent:cpp2web"` | `Index.ets:9` | IPC event constant |

## E. Built-in Skills (`/commands`)

| Command | Description | Parameters | Category |
|---------|-------------|------------|----------|
| `/search` | Search codebase with ripgrep patterns | `pattern`: string (required), `path`: string (optional, default: workspace root) | Code |
| `/scan` | Scan directory and build RAG index | `path`: string (required) | Code |
| `/goal` | Set or view agent goal | `description`: string (required) | Agent |
| `/file` | Read or write files in sandbox workspace | `action`: "read"\|"write" (required), `path`: string (required), `content`: string (optional, write-only) | File |
| `/session` | Manage chat sessions (list/new/switch/delete) | `action`: "list"\|"new"\|"switch"\|"delete" (required), `name`: string (optional) | Session |

## F. Host IO Callbacks (Rust → ArkTS)

| Callback Type | Signature | Route | Purpose |
|---------------|-----------|-------|---------|
| `post_fn` | `extern "C" fn(*const c_char, *const c_char) -> *mut c_char` | → `post_fn_proxy` → IO tsfn → `SystemIoImpl.httpPost()` | Blocking HTTP POST for LLM API calls |
| `stream_post_fn` | `extern "C" fn(*const c_char, *const c_char, extern "C" fn(*const c_char, u8)) -> bool` | → routes through registered `STREAM_CB` | Streaming HTTP POST stub |
| `free_str_fn` | `extern "C" fn(*mut c_char)` | → `free_str_fn_proxy` → `free()` | Free host-allocated strings |
| `STREAM_CB` | `Mutex<Option<extern "C" fn(*const c_char, u8)>>` | Rust thread → `OnChunkBridge` → stream tsfn → `DeliverChunkToJS` | Stream chunk delivery to ArkTS |

### Stream Event Types
| Event Type | Meaning |
|------------|---------|
| `0` | Data chunk — `chunk_data` is chunk JSON |
| `1` | Stream complete (empty data = done) |
| `2` | Error — `chunk_data` is error JSON |

## G. Session CRUD API

All via JSON action router. Storage: `<files_dir>/sessions/<id>.json` with `index.json`.

| Operation | Action String | Parameters | Returns |
|-----------|--------------|------------|---------|
| Create | `"create_session"` | `{"title": "<title>"}` | `Session` (meta + messages[]) |
| List | `"list_sessions"` | `{}` | `Vec<SessionMeta>` (id, title, dates, count) |
| Load | `"load_session"` | `{"session_id": "<id>"}` | `Session` (full object with messages) |
| Save | `"save_session"` | Full `Session` JSON | Status confirmation |
| Delete | `"delete_session"` | `{"session_id": "<id>"}` | Status confirmation |

## H. MCP Bridge API (Rust internal)

| Method | Signature | Description |
|--------|-----------|-------------|
| `McpBridge::new()` | `() -> McpBridge` | Create empty bridge |
| `register_server(server)` | `(McpServer) -> ()` | Add MCP server config |
| `start_server(name)` | `(&str) -> Result<(), String>` | Launch server subprocess |
| `list_tools(server)` | `(&str) -> Result<Vec<McpTool>, String>` | Send `tools/list` JSON-RPC request |
| `call_tool(server, tool, args)` | `(&str, &str, &Value) -> Result<Value, String>` | Send `tools/call` JSON-RPC request |
| `stop_server(name)` | `(&str) -> Result<(), String>` | Kill subprocess |

Protocol: JSON-RPC 2.0 over stdio (stdin/stdout). Supported methods: `tools/list`, `tools/call`.

## Endpoint Summary

| Interface | Count | Protocol |
|-----------|-------|----------|
| C FFI exports | 9 | C ABI (direct function call) |
| JSON actions | 14 | JSON over C FFI / NAPI |
| NAPI bridge functions | 7 | NAPI (HarmonyOS native interface) |
| ArkTS public exports | 9 | ArkTS module import |
| Built-in /commands | 5 | Chat text parsing |
| Host IO callbacks | 4 | C function pointer injection |
| Session CRUD | 5 | JSON action router |
| MCP Bridge | 6 | JSON-RPC 2.0 over stdio |
| **Total** | **59** | |

### Issue Endpoints

| Status | Count | Details |
|--------|-------|---------|
| WASM-only | 1 | `vfs_write_file` — returns error on non-wasm32 targets |
| Platform-gated | 2 | `search`, `scan_dir` — excluded from wasm32 builds |
| Stub/Partial | 1 | `stream_post_fn` in C++ — currently routes through registered STREAM_CB instead of using the passed on_chunk parameter directly |

