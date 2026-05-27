---
generated_at: 2026-05-27
cycles_found: 0
stub_modules: 7
choke_points: 3
---

# Deep Call Graph Summary

## Architecture: Strict DAG — No Circular Dependencies

The project maintains a clean layered DAG (Directed Acyclic Graph) across all three layers. All dependency arrows point in one direction with no cycles detected.

```
ArkTS UI → C++ NAPI → Rust FFI → JSON Router → Agent/Tools Modules
```

## Module Health Scores

### Rust Core
| Module | Score | Status | Notes |
|--------|-------|--------|-------|
| `agent/abort.rs` | 9/10 | Integrated | Minimal, single AtomicBool, used by 4 modules |
| `agent/skills.rs` | 9/10 | Integrated | Clean registry pattern, no internal deps |
| `agent/session.rs` | 8/10 | Integrated | Well-structured CRUD, file-based persistence |
| `agent/search.rs` | 8/10 | Integrated | Thin wrapper around ripgrep crates |
| `types/message.rs` | 8/10 | Integrated | Clean enum types, serde-tagged ContentPart |
| `agent/chat.rs` | 7/10 | Integrated | Single-function monolith, ureq with SSE parsing |
| `agent/rag.rs` | 7/10 | Integrated | Simple BM25 heuristic index, multi-threaded scan |
| `ffi.rs` | 7/10 | Integrated | Clear C ABI, but String return requires manual free |
| `sandbox/validate.rs` | 7/10 | Integrated | Stream cb + network/file tests, critical for streaming |
| `json_router.rs` | 6/10 | Integrated | 6 internal deps, monolithic dispatch function, manual JSON parsing per action |
| `agent/context.rs` | 6/10 | Stub (unused) | Token estimator defined but never called |
| `agent/pipeline.rs` | 6/10 | Stub (unused) | SseMerger + FrameThrottler defined but never imported |
| `agent/loop_engine.rs` | 5/10 | Stub (unused) | Full ReAct loop with SHA-256 detection, no consumer |
| `tools/file.rs` | 5/10 | Stub (unused) | Sandbox file ops defined but never imported |
| `tools/mcp.rs` | 5/10 | Stub (unused) | Full JSON-RPC MCP client, no consumer |
| `tools/subagent.rs` | 5/10 | Stub (unused) | Sub-agent runner with progress callbacks, no consumer |
| `tools/web.rs` | 5/10 | Stub (unused) | DDG search + HTML fetch, no consumer |

**Average Rust score: 6.5/10** (docked for 7/17 stub modules = 41%)

### ArkTS UI
| Component | Score | Status | Notes |
|-----------|-------|--------|-------|
| `MarkdownParser` | 9/10 | Integrated | Pure function, zero imports |
| `ArkContentPart` | 9/10 | Integrated | Clean @ObservedV2 model |
| `InputBar` | 9/10 | Integrated | Clean @ComponentV2, all data via @Param |
| `ToolCard` | 9/10 | Integrated | Clean @ComponentV2, all data via @Param |
| `MarkdownView` | 8/10 | Integrated | Single dependency on MarkdownParser |
| `ArkMessage` | 8/10 | Integrated | Clean model, depends on ArkContentPart |
| `PhraseLoader` | 8/10 | Integrated | Singleton with good caching |
| `ConversationList` | 8/10 | Integrated | Clean @ComponentV2, all data via @Param |
| `ThinkingPanel` | 8/10 | Integrated | Good @Monitor usage for lifecycle |
| `RustAgentBridge` | 7/10 | Integrated | Critical singleton, retry logic, offline detection |
| `MessageBubble` | 7/10 | Integrated | Clean part-type dispatcher |
| `ProviderLoader` | 7/10 | Integrated | 3-tier loading with good fallback |
| `SettingsPage` | 7/10 | Integrated | Clean form with preferences persistence |
| `CommandPalette` | 6/10 | Stub (unused in ChatView) | Full implementation, not wired to ChatView |
| `SystemIoImpl` | 5/10 | Stub (unused) | HTTP/file IO defined, not wired to bridge callback |
| `SseStreamController` | 5/10 | Stub (deprecated) | Imported but unused — Rust ureq handles SSE |
| `ChatView` | 5/10 | Integrated | Monolithic (~590 lines), 7 imports, handles too many concerns |

**Average ArkTS score: 7.2/10** (docked for ChatView monolith + 2 unused modules)

## Blast Radius Analysis

### Critical Impact (blast radius ≥ 15)
| Node | Direct | Indirect | Total | Affected Modules |
|------|--------|----------|-------|-----------------|
| `ffi.rs` | 2 | 17 | **19** | wasm.rs, node.rs, native_bridge.cpp, all ArkTS UI (17 components) |
| `native_bridge.cpp` | 1 | 17 | **18** | ffi.rs → all Rust, all ArkTS UI via RustAgentBridge |
| `RustAgentBridge.ets` | 1 | 16 | **17** | ChatView → all 16 child UI components |
| `ChatView.ets` | 5 | 12 | **17** | InputBar, ConversationList, MessageBubble, SettingsPage, SseStreamController + all child components |
| `json_router.rs` | 1 | 14 | **15** | ffi.rs → native_bridge → all ArkTS + 6 internal Rust modules |

### High Impact (blast radius 5-14)
| Node | Direct | Indirect | Total |
|------|--------|----------|-------|
| `agent/abort.rs` | 4 | 6 | **10** |
| `types/message.rs` | 3 | 4 | **7** |
| `agent/chat.rs` | 1 | 3 | **4** |

### Low Impact (blast radius < 5)
All stub modules (context, pipeline, loop_engine, file, mcp, subagent, web) have blast radius 0-2 since nothing imports them yet.

## Choke Points (Single Points of Failure)

1. **`json_router.rs`** — Every ArkTS action passes through `dispatch()`. A panic here kills all functionality.
2. **`native_bridge.cpp`** — Only cross-boundary bridge. All 7 NAPI exports, 2 tsfn channels (stream + IO). Thread-safety bugs crash the entire app.
3. **`ffi.rs`** — C ABI contract. Any signature change breaks WASM, Node.js, and HarmonyOS simultaneously.

## Cross-Layer Call Chain (Chat Message End-to-End)

```
User types "Hello" in InputBar
  → ChatView.handleSend("Hello")
    → bridge.call('chat', JSON.stringify({messages}))
    → NAPI → native_bridge.cpp → AgentCall()
    → rust_agent_call("chat", json) → ffi.rs
    → json_router::dispatch("chat", args)
      → Parse AgentRequest::ChatStream
      → Read AGENT_CONFIG (api_key, base_url, model)
      → thread::spawn:
        → chat_completion_ureq(config, messages, on_chunk)
          → ureq POST {base_url}/chat/completions (SSE)
          → For each SSE data: chunk:
            → on_chunk(delta_json) → STREAM_CB(data_cstr, event_type=0)
              → OnChunkBridge() → napi_threadsafe_function
                → DeliverChunkToJS() → JS stream callback
                  → ChatView.applyDelta(delta)
                    → @Trace update → ArkUI re-render
          → On done: STREAM_CB("", event_type=1)
            → set isStreaming=false, save session
```

## Cross-Module Heat Map

| From \ To | ffi | json_router | chat | session | skills | abort | types | sandbox |
|-----------|-----|-------------|------|---------|--------|-------|-------|---------|
| **ffi.rs** | - | **X** (call) | - | **X** (init) | - | - | - | - |
| **json_router** | - | - | **X** (dispatch) | **X** (dispatch) | **X** (dispatch) | **X** (set) | **X** (parse) | **X** (read cb) |
| **ChatView** | - | - | - | - | - | **X** (abort) | - | - |

## Recommendations

1. **Integrate stub modules**: 7 Rust modules (41%) are complete but unused. Wire `loop_engine` into `json_router::dispatch("chat")` to enable ReAct agent behavior. Wire `tools/*` into the agent loop.
2. **Decompose ChatView**: At ~590 lines handling 7 concerns, split into: `SessionManager` (session CRUD), `StreamHandler` (delta application), `BridgeInitializer` (init lifecycle).
3. **Replace json_router match with registry**: The 300-line match statement could become a `HashMap<String, Box<dyn ActionHandler>>` for extensibility.
4. **Add integration tests for cross-boundary flow**: No tests cover the full ArkTS→NAPI→FFI→dispatch→callback→ArkTS path.
5. **Wire CommandPalette**: Component is fully implemented but `showPalette` is never set to true in ChatView.

