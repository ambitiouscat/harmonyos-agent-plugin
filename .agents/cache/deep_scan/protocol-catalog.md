---
generated_at: 2026-05-27
protocols:
  - HTTP REST (LLM API)
  - HTTP REST (Provider Directory)
  - SSE Streaming (LLM output)
  - JSON-RPC 2.0 (MCP)
  - C FFI (extern "C" ABI)
  - NAPI (HarmonyOS Native API)
  - NAPI (Node.js)
  - WASM Bindings
  - EventHub IPC (String-Keyed)
  - Stream Callback IPC
  - IO Proxy IPC (blocking callback)
serialization_formats:
  - JSON (exclusive — no binary formats)
auth_mechanisms:
  - API Key via Bearer Token
  - None (public endpoints)
---

# Protocol Catalog

## 1. HTTP REST Endpoints

### 1.1 LLM Chat Completions API

| Field | Value |
|-------|-------|
| Transport | HTTP/HTTPS (TLS via rustls) |
| Method | POST |
| URL Pattern | `{base_url}/chat/completions` |
| Client | Rust `ureq` v3 (rustls + json features) |
| Headers | `Authorization: Bearer {api_key}`, `Content-Type: application/json` |
| Body | `{ model, messages, stream: true, max_tokens }` |
| Timeout | 30s connect, 120s read |
| Source | `rust_core/.../agent/chat.rs:11-104` |

### 1.2 Model Provider Directory API

| Field | Value |
|-------|-------|
| Transport | HTTPS |
| Method | GET |
| URL | `https://models.dev/api/json` |
| Client | HarmonyOS `@kit.NetworkKit` (`http.createHttp()`) |
| Fallback | Embedded rawfile → 7-day cached → remote |
| Source | `hmosagent/.../provider/ProviderLoader.ets:151-172` |

### 1.3 ArkTS HTTP POST (IO Proxy)

| Field | Value |
|-------|-------|
| Transport | HTTPS (HarmonyOS @kit.NetworkKit) |
| Method | POST |
| Headers | `Content-Type: application/json` |
| Purpose | Blocking IO proxy for Rust `post_fn` callback; also direct HTTP |
| Source | `hmosagent/.../tools/SystemIoImpl.ets:18-46` |

## 2. SSE (Server-Sent Events) — Streaming

### 2.1 Rust SSE Producer

| Field | Value |
|-------|-------|
| Delimiter | `\n\n` (double newline) event boundary |
| Data Prefix | `data: ` |
| Terminal Signal | `data: [DONE]` |
| Chunk Format | `{ "choices": [{"delta": {"content": "..."}}] }` |
| Delta Rewrapping | Rust extracts `choices[0].delta.content` → `{"type":"text","text":content}` |
| Inter-Chunk Delay | 20ms `thread::sleep` (allows ArkUI render loop to paint) |
| Source | `rust_core/.../agent/chat.rs:52-101` |

### 2.2 SSE Reconnection & Throttling

| Component | Detail |
|-----------|--------|
| `SseMerger` | Accumulates partial JSON from `data:` lines, emits when `serde_json::from_str` succeeds |
| `ReconnectConfig` | max_retries=3, base_delay_ms=500, max_delay_ms=5000, exponential backoff |
| `FrameThrottler<T>` | Batches deltas into ~16ms windows aligned to render frames |
| Source | `rust_core/.../agent/pipeline.rs` |

### 2.3 ArkTS SSE Consumer (Fallback)

| Field | Value |
|-------|-------|
| Class | `SseStreamController` |
| Mechanism | Splits body on `\n\n`, iterates `data:` lines, `JSON.parse` each |
| Delivery | `setTimeout` chain at 20ms per chunk, cancellable |
| Status | **Deprecated** — imported but unused; Rust ureq handles SSE |
| Source | `hmosagent/.../stream/SseStreamController.ets` |

## 3. JSON-RPC 2.0 (MCP Protocol)

| Field | Value |
|-------|-------|
| Transport | stdio (child process stdin/stdout) |
| Protocol | JSON-RPC 2.0 |
| Line Delimiter | Newline-delimited JSON (`\n`) on stdout |
| Methods | `tools/list`, `tools/call` |
| Types | `JsonRpcRequest { jsonrpc, id, method, params }`, `JsonRpcResponse { result, error }` |
| Server Config | `McpServer { name, command, args, env }` |
| Lifecycle | spawn → `tools/list` → `tools/call` → kill |
| Status | **Stub** — fully implemented but no consumer imports it |
| Source | `rust_core/.../tools/mcp.rs` |

## 4. C FFI (Rust C ABI Exports)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `rust_agent_init` | `(SystemCallbacks) -> bool` | Inject host IO callbacks |
| `rust_agent_init_session` | `(*const c_char) -> bool` | Initialize session file directory |
| `rust_agent_call` | `(*const c_char, *const c_char) -> *mut c_char` | Unified JSON action dispatcher |
| `rust_agent_free_str` | `(*mut c_char) -> void` | Free string from rust_agent_call |
| `rust_agent_search` | `(*const c_char, *const c_char) -> *mut c_char` | In-process ripgrep (not wasm32) |
| `rust_agent_scan_dir` | `(*const c_char) -> *mut c_char` | Build BM25 RAG index (not wasm32) |
| `rust_agent_register_stream_cb` | `(fn(*const c_char, u8)) -> void` | Register stream chunk callback |
| `test_network` | `() -> bool` | TCP connect to 8.8.8.8:53 |
| `test_file` | `(*const c_char) -> bool` | Write+delete temp file sandbox test |

### SystemCallbacks Struct
```c
typedef struct {
    char* (*post_fn)(const char* url, const char* body);
    bool  (*stream_post_fn)(const char* url, const char* body, void (*on_chunk)(const char*, uint8_t));
    void  (*free_str_fn)(char* ptr);
} SystemCallbacks;
```
Sources: `ffi.rs:20-27`, `agent_core.h:23-27`

## 5. NAPI (HarmonyOS Native API)

| Field | Value |
|-------|-------|
| Module Name | `native_bridge` |
| Exports | `initAgent`, `initAgentWithIo`, `agentCall`, `testNetwork`, `testFile`, `search`, `scanDir` |
| Stream Delivery | `napi_threadsafe_function` (`g_stream_tsfn`) → `DeliverChunkToJS` |
| IO Proxy | `napi_threadsafe_function` (`g_io_tsfn`) → `DeliverIoToJS` + `mutex` + `condition_variable` |
| Event Types | 0=data, 1=done, 2=error |
| Source | `hmosagent/.../cpp/native_bridge.cpp` |

## 6. WASM Bindings

| Field | Value |
|-------|-------|
| Feature | `wasm` (wasm-bindgen + js-sys) |
| Exports | `wasm_agent_init(on_chunk, on_io)`, `wasm_agent_call(action, json_args)` |
| JS Bridge | `OnceLock<js_sys::Function>` stores callbacks; `wasm_on_chunk_bridge` converts C string to JsValue |
| Source | `rust_core/.../agent/wasm.rs` |

### Node.js NAPI Binding
| Field | Value |
|-------|-------|
| Feature | `node` (napi + napi-derive) |
| Exports | `node_agent_init(config_json)`, `node_agent_call(action, args)` |
| Difference | Direct Rust IO (ureq + std::fs), no JS callbacks needed |
| Source | `rust_core/.../agent/node.rs` |

## 7. Cross-Thread IPC

### 7.1 EventHub IPC (Godot ↔ ArkTS ↔ Web)

| Field | Value |
|-------|-------|
| Constants | `CMD_WEB2CPP = "agent:web2cpp"`, `EVENT_CPP2WEB = "agent:cpp2web"` |
| Transport | HarmonyOS `EventHub` (string-keyed event bus) + `WebMessagePort` |
| Flow (C++→Web) | `Godot send_data_to_browser()` → tsfn → ArkTS → `EventHub.emit(CPP2WEB)` → `MessagePort.postMessageEvent()` |
| Flow (Web→C++) | `MessagePort.onMessageEvent()` → `EventHub.emit(WEB2CPP)` → `plugin.frontSlot(data)` |
| Source | `Index.ets:8-9`; spec at `openspec/changes/archive/.../event-hub-bridge/spec.md` |

### 7.2 Stream Callback IPC (Rust → ArkTS)

| Field | Value |
|-------|-------|
| Registration | `rust_agent_register_stream_cb(callback)` → stored in `STREAM_CB: Mutex` |
| Producer | Rust worker thread spawned by `json_router::dispatch("chat")` |
| Consumer | C++ `OnChunkBridge` → `napi_threadsafe_function` → ArkTS main thread |
| Source | `sandbox/validate.rs:7-17`, `native_bridge.cpp:150-169` |

### 7.3 IO Proxy IPC (Rust → ArkTS, Blocking)

| Field | Value |
|-------|-------|
| Flow | Rust `post_fn(url, body)` → C++ `post_fn_proxy` → tsfn → ArkTS `SystemIoImpl.httpPost()` → response → `condition_variable.notify()` → Rust resumes |
| Synchronization | `std::mutex` + `std::condition_variable` (Rust thread blocks until ArkTS responds) |
| Source | `native_bridge.cpp:67-108` |

## 8. Message Serialization Formats

| Format | Usage | Prevalence |
|--------|-------|------------|
| **JSON** | All cross-boundary communication, SSE deltas, session persistence, provider registry, AgentRequest/Response, ContentPart, MCP JSON-RPC | **Exclusive** |
| Protobuf | — | Not used |
| MessagePack | — | Not used |
| CBOR | — | Not used |
| Bincode | — | Not used |
| FlatBuffers | — | Not used |
| Custom binary | — | Not used |

**Conclusion**: JSON (via `serde_json` in Rust, `JSON.parse`/`JSON.stringify` in ArkTS) is the **exclusive** serialization format across the entire project.

## 9. Authentication Mechanisms

### 9.1 API Key via Bearer Token (Primary)
| Field | Value |
|-------|-------|
| Header | `Authorization: Bearer {api_key}` |
| Storage | HarmonyOS `Preferences` API (OS-level encrypted) |
| Input | `TextInput` with `InputType.Password` in SettingsPage |
| Transmission | ArkTS → Rust via `bridge.call("configure", {api_key, ...})` → stored in `AGENT_CONFIG` RwLock |
| Source | `SettingsPage.ets:202`, `chat.rs:16`, `ChatView.ets:99-111` |

### 9.2 No Authentication (Public Endpoints)
- Provider directory (`https://models.dev/api/json`): no auth headers
- MCP servers: no auth (stdio subprocess, trust via process isolation)

### 9.3 Not Used
- JWT: no token parsing, no refresh flows
- OAuth 2.0: no authorization code, no PKCE
- Session/Cookie: no cookie storage
- Basic Auth: no `Basic` header patterns

## 10. Protocol Confidence Matrix

| # | Protocol | Location | Confidence | Evidence |
|---|----------|----------|------------|----------|
| 1 | HTTP REST (LLM API) | `agent/chat.rs` | **HIGH** | `ureq::post()`, Bearer auth, explicit URL |
| 2 | HTTP REST (Provider) | `ProviderLoader.ets` | **HIGH** | `http.request(REMOTE_URL)`, GET method |
| 3 | SSE Streaming | `chat.rs`, `pipeline.rs` | **HIGH** | `data:` prefix, `[DONE]`, `\n\n` delimiters, 20ms delay |
| 4 | JSON-RPC 2.0 (MCP) | `tools/mcp.rs` | **HIGH** | Explicit JsonRpcRequest/Response structs, `jsonrpc: "2.0"` |
| 5 | C FFI (extern "C") | `ffi.rs`, `agent_core.h` | **HIGH** | `#[no_mangle] pub extern "C"`, CString, `#[repr(C)]` |
| 6 | NAPI (HarmonyOS) | `native_bridge.cpp` | **HIGH** | `napi_threadsafe_function`, `napi_module_register` |
| 7 | NAPI (Node.js) | `node.rs` | **HIGH** | `#[napi]` attribute, `napi_derive` |
| 8 | WASM Bindings | `wasm.rs` | **HIGH** | `#[wasm_bindgen]`, `js_sys::Function` |
| 9 | EventHub IPC | `Index.ets`, spec.md | **HIGH** | `CMD_WEB2CPP`/`EVENT_CPP2WEB` constants, spec doc |
| 10 | Web MessagePort | spec.md (external) | **MEDIUM** | Referenced in design docs; implementation in i3d544 host project |
| 11 | IO Proxy IPC | `native_bridge.cpp` | **HIGH** | Mutex+condition_variable blocking pattern |
| 12 | Stream Callback IPC | `validate.rs` | **HIGH** | `STREAM_CB: Mutex<Option<extern "C" fn>>` |
| 13 | WebSocket | — | **NONE** | No `ws://` or `wss://` found |
| 14 | Binary Formats | — | **NONE** | No Protobuf, MessagePack, CBOR, or bincode deps |

## Platform Protocol Matrix

| Platform | HTTP | SSE | C FFI | NAPI | WASM | IPC |
|----------|------|-----|-------|------|------|-----|
| HarmonyOS | ✓ (via Rust ureq or IO proxy) | ✓ | ✓ | ✓ | — | ✓ (EventHub) |
| Node.js | ✓ (native ureq) | ✓ | — | ✓ (napi-rs) | — | — |
| WASM/Web | ✓ (JS fetch) | ✓ | ✓ (via wasm-bindgen) | — | ✓ | — |
| Godot Plugin | ✓ (via IO proxy) | ✓ | ✓ | — | — | ✓ (EventHub + MessagePort) |

