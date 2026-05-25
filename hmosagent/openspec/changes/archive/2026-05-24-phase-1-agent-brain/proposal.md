## Why

Phase 0 established the physical C ABI channel (Rust → C++ NAPI → ArkTS). Phase 1 fills that channel with a complete Rust-side AI agent brain — ReAct loop engine, SSE streaming pipeline, context window management, in-process code search, and BM25 RAG — while also closing the IO injection gap so the Rust core can make real HTTP calls through ArkTS.

## What Changes

- **REFACTOR** message model: Replace Phase 0 simplified `Message { role, content }` with polymorphic `ContentPart` enum (5 variants: text, reasoning, tool_call, tool_result, finish)
- **NEW** `agent/loop_engine.rs`: ReAct controller with SHA-256 sliding-window loop detection + MAX_ITERATIONS=30 hard cap
- **NEW** `agent/pipeline.rs`: SSE delta merge state machine + 16ms frame-aligned throttler
- **NEW** `agent/context.rs`: Token budget manager with 80% compaction threshold, system-prompt preservation, tool-output trimming
- **NEW** `agent/search.rs`: In-process ripgrep via `ignore` + `grep-*` crates (sandbox-compliant, no subprocess)
- **NEW** `agent/rag.rs`: BM25 Level-0 inverted index with multi-threaded directory scanning and chunking
- **NEW** FFI: `rust_agent_search`, `rust_agent_scan_dir` exposed via C ABI
- **ENHANCE** `native_bridge.cpp`: `post_fn_proxy` (condition-variable blocking IO), `stream_post_fn_proxy`, `initAgentWithIo` (dual tsfn), `search` and `scanDir` NAPI exports
- **NEW** `SystemIoImpl.ets`: ArkTS-side HTTP (`@ohos.net.http`) and filesystem (`@ohos.file.fs`) implementations
- **REFACTOR** `RustAgentBridge.ets`: Added `initWithIo`, `search`, `scanDir` methods
- **UPDATE** `Index.ets`: Scrollable 6-button test page with Phase 1 Search Files + Scan Dir buttons
- **TEST** Unified `[TEST]` hilog prefix across all NAPI test functions

## Capabilities

### New Capabilities
- `search-engine`: In-process ripgrep file search, sandbox-compliant, `.gitignore`-aware, multi-threaded
- `rag-engine`: BM25 keyword inverted index with async directory scanning, chunking, and query
- `io-proxy`: Condition-variable-based blocking C→ArkTS IO bridge for Rust's `post_fn`

### Modified Capabilities
- `rust-agent-core`: Extended message model (ContentPart), new agent sub-modules (loop_engine/pipeline/context/search/rag), new FFI exports (rust_agent_search/rust_agent_scan_dir)
- `napi-native-bridge`: Enhanced InitAgent wiring with real IO callbacks, new `initAgentWithIo` with dual tsfn, new `search`/`scanDir` NAPI exports
- `harmonyos-project-scaffold`: New `SystemIoImpl.ets`, refactored `RustAgentBridge.ets`, updated `Index.ets`

## Impact

- **Rust**: `agent_core` Cargo.toml (+sha2, +ignore, +grep-regex, +grep-searcher deps), 6 new source files, 2 modified
- **C++**: `native_bridge.cpp` (+~120 lines: post_fn_proxy, initAgentWithIo, search, scanDir), `agent_core.h` (+2 declarations)
- **ArkTS**: `RustAgentBridge.ets` (refactor), `SystemIoImpl.ets` (new), `Index.ets` (update)
- **Build**: libagent_core.a size increase from new crate dependencies; both archs (arm64-v8a, x86_64) verified
- **Tests**: 32 Rust unit tests pass, Phase 0 retro-compatibility verified on emulator
