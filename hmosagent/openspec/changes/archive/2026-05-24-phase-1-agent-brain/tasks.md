## 1. Rust Core — Message Model

- [x] 1.1 REFACTOR `types/message.rs`: Replace simplified Message with polymorphic ContentPart enum (text, reasoning, tool_call, tool_result)
- [x] 1.2 Add ContentPart unit tests (round-trip, tagged serialization, tool_result error flag)
- [x] 1.3 Verify Phase 0 AgentRequest untagged deserialization still works

## 2. Rust Core — Agent Engine

- [x] 2.1 NEW `agent/loop_engine.rs`: ReAct controller, SHA-256 sliding window (size 5), MAX_ITERATIONS=30
- [x] 2.2 NEW `agent/pipeline.rs`: SseMerger + FrameThrottler (16ms window)
- [x] 2.3 NEW `agent/context.rs`: Token budget manager, 80% compaction threshold, system prompt preservation
- [x] 2.4 Unit tests: loop detection, max iterations, SSE merge, frame throttling, token estimation, compaction

## 3. Rust Core — Search & RAG

- [x] 3.1 NEW `agent/search.rs`: InProcessSearcher using `ignore` + `grep-regex` + `grep-searcher` crates
- [x] 3.2 NEW `agent/rag.rs`: RagIndex with multi-threaded scanning, paragraph chunking, BM25 keyword search
- [x] 3.3 Unit tests: temp-dir search, rag index scan+search, tokenize filter

## 4. Rust Core — FFI & Dependencies

- [x] 4.1 Add `sha2`, `ignore`, `grep-regex`, `grep-searcher` to Cargo.toml
- [x] 4.2 NEW FFI: `rust_agent_search(dir, pattern)` and `rust_agent_scan_dir(dir)` in ffi.rs
- [x] 4.3 Add `SearchMatch` Serialize derive for JSON output
- [x] 4.4 Update `lib.rs`: declare `pub mod agent`
- [x] 4.5 Full test suite (32 tests pass), 0 warnings

## 5. C++ NAPI Bridge — IO Proxy

- [x] 5.1 UPDATE `agent_core.h`: Add `rust_agent_search`, `rust_agent_scan_dir` declarations
- [x] 5.2 ENHANCE `native_bridge.cpp`: Add `post_fn_proxy` (condition-variable blocking), `stream_post_fn_proxy`, `free_str_fn_proxy`
- [x] 5.3 ENHANCE `native_bridge.cpp`: Add `initAgentWithIo` (dual tsfn: stream + IO)
- [x] 5.4 ENHANCE `native_bridge.cpp`: Add `search`, `scanDir` NAPI exports
- [x] 5.5 ENHANCE `native_bridge.cpp`: Add `[TEST]` hilog prefix to all test functions (agentCall, testNetwork, search, scanDir)
- [x] 5.6 Fix forward declaration for `JsStringToCpp` used in `DeliverIoToJS`

## 6. ArkTS — Host IO & UI

- [x] 6.1 NEW `SystemIoImpl.ets`: httpPost, readFile, writeFile, fileExists using @ohos.net.http + @ohos.file.fs
- [x] 6.2 REFACTOR `RustAgentBridge.ets`: Add initWithIo(onChunk, onIo), search(dir, pattern), scanDir(dir) methods
- [x] 6.3 UPDATE `Index.ets`: Scrollable 6-button test page (Phase 0 buttons + Search Files + Scan Dir)

## 7. Cross-Compile & Deploy

- [x] 7.1 Build x86_64 + aarch64 release .a libraries
- [x] 7.2 Copy libagent_core.a to hmosagent/libs/{arch}/
- [x] 7.3 hvigorw assembleHap: BUILD SUCCESSFUL (dual ABI)
- [x] 7.4 Deploy to emulator (install + start)
- [x] 7.5 Verify Phase 0 retro-compatibility: Init/Ping/Network/Stream all pass
- [x] 7.6 Verify Phase 1 functions: search/scanDir return proper JSON (Permission denied expected for `.` path on emulator)

## 8. Documentation & OpenSpec

- [x] 8.1 Review Phase 1 implementation plan (4 rounds) → approved
- [x] 8.2 Git commit with descriptive message
- [x] 8.3 OpenSpec artifacts: proposal, design, specs (3 new + 3 delta), tasks
