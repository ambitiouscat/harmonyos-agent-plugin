---
generated_at: 2026-05-27
stack:
  - ArkTS/ArkUI (HarmonyOS SDK 6.0.2)
  - Rust 2021 (Cargo workspace, staticlib+cdylib+rlib)
  - C++17 (NAPI bridge, CMake)
  - C (musl compatibility shim)
  - Bash (build orchestration)
  - JSON5 (HarmonyOS config)
  - CMake (native build)
---

# Technology Stack

## Languages

| Language | Usage | File Count |
|----------|-------|------------|
| ArkTS / ArkUI | HarmonyOS UI — chat, settings, Markdown rendering, state management | 28 .ets files |
| Rust 2021 | Core agent engine — LLM, ReAct loop, session CRUD, RAG, search, MCP | ~15 .rs files |
| C++17 | NAPI bridge — thread-safe ArkTS↔Rust communication | 2 .cpp files |
| C | OHOS musl compatibility shim (missing symbols) | 1 .c file |
| Bash | Cross-compilation & build orchestration | 5 .sh files |
| Batch | Windows-side Rust builds (WASM, Node) | 2 .bat files |

## Build Systems

### HarmonyOS (hmosagent/)
- **Tool**: Hvigor (HarmonyOS native build system)
- **SDK**: targetSdkVersion 6.0.2(22), compatibleSdkVersion 5.0.0(12)
- **Runtime**: HarmonyOS (not OpenHarmony)
- **Mode**: stageMode
- **Target Devices**: phone, 2in1
- **ABI Filters**: arm64-v8a, x86_64
- **Package Manager**: OHPM

### Rust (rust_core/)
- **Tool**: Cargo (edition 2021, resolver v2)
- **Crate Types**: `staticlib` (OHOS), `cdylib` (WASM), `rlib` (tests)
- **Cross-compilation**: OHOS NDK via WSL2 (aarch64/x86_64-unknown-linux-musl)
- **Release**: opt-level="s", LTO, stripped, panic="abort"
- **FFI**: cbindgen → `agent_core.h`

### Native Bridge
- **Tool**: CMake + Ninja (triggered by Hvigor externalNativeOptions)
- **Output**: `libnative_bridge.so`
- **Links**: `libagent_core.a` + `ace_napi` + `hilog`

## Rust Dependencies

### Runtime
| Crate | Version | Purpose |
|-------|---------|---------|
| serde/serde_json | 1.x | JSON serialization |
| sha2 | 0.10 | SHA-256 (loop detection) |
| ureq | 3.x (rustls+json) | HTTP client for LLM API |
| ignore | 0.4 | Gitignore-aware directory traversal |
| grep-regex | 0.1 | In-process regex search |
| grep-searcher | 0.1 | In-process ripgrep search |

### Platform Optional
| Feature | Crate | Platform |
|---------|-------|----------|
| wasm | wasm-bindgen, js-sys | WebAssembly |
| node | napi, napi-derive | Node.js addon |

### Dev
| Crate | Purpose |
|-------|---------|
| tempfile | Temporary directories for tests |

## HarmonyOS Dependencies
- **Runtime**: Zero external dependencies (SDK built-ins only)
- **Dev**: @ohos/hypium (testing), @ohos/hamock (mocking)

## Key Architectural Decisions

1. **JSON Action Router**: All ArkTS→Rust calls go through `rust_agent_call(action, json_args)` dispatching ~20 actions
2. **NAPI Thread-Safe Bridge**: `napi_threadsafe_function` for Rust→ArkTS callbacks (SSE streaming)
3. **Host IO Callbacks**: Rust core does not call OS directly; IO goes through host callbacks (post_fn, stream_post_fn)
4. **Platform Abstraction**: `features = ["wasm"]` / `features = ["node"]` gate platform-specific adapters
5. **File-Based Sessions**: Sessions stored as JSON files, managed by SessionManager
6. **ReAct Loop**: Think→Act→Observe cycle with SHA-256 loop detection (5-entry sliding window, 30-iteration hard limit)

