---
generated_at: 2026-05-27
module_count: 9
---

# Modules Catalog

## Module Index

| ID | Module | Path | Language | Cache Level | Purpose |
|----|--------|------|----------|-------------|---------|
| M1 | entry | `hmosagent/entry/` | ArkTS | L3 | Shell HAP — app lifecycle & main page |
| M2 | RustAgentBridge | `hmosagent/.../ets/RustAgentBridge.ets` | ArkTS | L4 | Singleton NAPI bridge to Rust core |
| M3 | UI Components | `hmosagent/.../ets/ui/` | ArkTS | L4 | Chat UI, Markdown, input, conversations |
| M4 | Settings | `hmosagent/.../ets/pages/SettingsPage.ets` | ArkTS | L3 | Provider/API key/model configuration |
| M5 | Provider | `hmosagent/.../ets/provider/ProviderLoader.ets` | ArkTS | L3 | 3-tier LLM provider list loading |
| M6 | Stream | `hmosagent/.../ets/stream/SseStreamController.ets` | ArkTS | L3 | SSE parser with UI-yield timers |
| M7 | NAPI Bridge | `hmosagent/.../cpp/` | C++17 | L3 | Thread-safe C++ ↔ Rust bridge |
| M8 | Rust Agent Core | `rust_core/agent_core/src/` | Rust | L4 | LLM chat, ReAct loop, session, RAG, tools |
| M9 | Build Scripts | `scripts/` | Bash | L2 | Cross-compile & HAP build orchestration |

## Module Details

### M1: entry (HAP Shell)
- **Files**: 2 (.ets EntryAbility + Index page)
- **Role**: App lifecycle management, loads main ChatView
- **Depends on**: M2 (RustAgentBridge via hmos_agent_core HAR)
- **Key classes**: `EntryAbility`, `Index`

### M2: RustAgentBridge (NAPI Bridge Singleton)
- **Files**: 1 main + related IO/SSE controllers
- **Role**: All Rust core communication; init, call, search, scanDir, session CRUD
- **Depends on**: M7 (NAPI C++ bridge)
- **Key features**: Exponential backoff retry, offline detection, abort support, IPC constants

### M3: UI Components
- **Files**: 14 .ets files in 6 subdirectories
- **Sub-modules**:
  - `chat/` — ChatView (~590 lines), MessageBubble, MarkdownView, MarkdownParser
  - `common/` — ThinkingPanel (spinner), PhraseLoader
  - `conversation/` — ConversationList (80% width drawer)
  - `input/` — InputBar (send/stop), CommandPalette (/command dropdown)
  - `models/` — ArkMessage, ArkContentPart (@ObservedV2 + @Trace)
  - `tools/` — ToolCard (status + output)
- **Depends on**: M2 (bridge), M6 (SSE controller)

### M4: Settings
- **Files**: 1 (SettingsPage.ets)
- **Role**: Provider selection, API key input (password field), model chooser, phrase style selector, base URL
- **Persistence**: HarmonyOS preferences API
- **Depends on**: M5 (ProviderLoader)

### M5: ProviderLoader
- **Files**: 1 (ProviderLoader.ets)
- **Role**: 3-tier LLM provider list loading (embedded fallback → 7-day cached → remote fetch)
- **Sorting**: Common providers (OpenAI, Anthropic, DeepSeek, etc.) sorted to top
- **Depends on**: Rawfile `providers.json`

### M6: SSE Stream Controller
- **Files**: 1 (SseStreamController.ets)
- **Role**: Parse `data: ...` SSE events, interleaved ~20ms timers for UI yield, cancelable
- **Depends on**: None (pure TS)

### M7: NAPI C++ Bridge
- **Files**: 3 (.cpp + .c + .h) + 1 CMakeLists.txt
- **Role**: Thread-safe marshaling between ArkTS main thread and Rust worker threads
- **Key mechanism**: `napi_threadsafe_function` for Rust → ArkTS callbacks
- **Depends on**: M8 (libagent_core.a)

### M8: Rust Agent Core
- **Files**: ~15 .rs source + 1 test file
- **Sub-modules**:
  - `ffi.rs` — 9 pub extern "C" functions
  - `json_router.rs` — ~20 action dispatch
  - `agent/chat.rs` — LLM HTTP SSE streaming
  - `agent/loop_engine.rs` — ReAct loop with loop detection
  - `agent/session.rs` — JSON file-based session CRUD
  - `agent/skills.rs` — 5 built-in commands (/search, /scan, /goal, /file, /session)
  - `agent/context.rs` — Token budget, 80% threshold compression
  - `agent/pipeline.rs` — SSE merger, frame throttler, reconnect config
  - `agent/rag.rs` — BM25 heuristic RAG with parallel indexing
  - `agent/search.rs` — In-process ripgrep search
  - `tools/file.rs` — Sandboxed file read/write/list
  - `tools/mcp.rs` — MCP client (stdio JSON-RPC)
  - `tools/subagent.rs` — Sub-agent runner (threaded)
  - `tools/web.rs` — Host-based web search/fetch
- **Tests**: ~35 unit tests + 7 integration tests
- **Depends on**: serde, serde_json, sha2, ureq, ignore, grep-*

### M9: Build Scripts
- **Files**: 5 .sh
- **Role**: Cross-compilation pipeline orchestration
- **Workflow**: WSL Rust cross-compile → HAP build via hvigor
- **Depends on**: DevEco Studio, OHOS NDK, Rust toolchain

