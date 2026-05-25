## Context

Phase 0 delivered a working C ABI bridge: `rust_agent_init` → `rust_agent_call` → JSON response, plus `napi_threadsafe_function` for streaming callbacks. The Rust core had a placeholder `Message { role, content }` model and stub action handlers (`ping`, `test_stream`).

Phase 1 transforms this scaffold into a real AI agent brain. The key constraint: all business logic MUST reside in Rust (for cross-platform portability to VS Code/WASM later), while ArkTS provides only UI rendering and platform IO.

## Goals / Non-Goals

**Goals:**
- Polymorphic message model (ContentPart) supporting text, reasoning, tool calls, and tool results
- ReAct loop controller with infinite-loop detection (SHA-256 sliding window + hard cap)
- SSE stream delta merging with 16ms frame-aligned throttling
- Context window management with automatic compaction at 80% threshold
- In-process ripgrep file search (sandbox-compliant, no subprocess)
- BM25 Level-0 RAG index with multi-threaded directory scanning
- Close the IO loop: Rust's `post_fn` blocks the calling thread until ArkTS delivers an HTTP response via condition variable

**Non-Goals:**
- SkillsRegistry (deferred to Phase 2)
- Vector embedding / ONNX RAG Level 1 (deferred)
- SubAgent runner (deferred to Phase 4)
- LLM summarisation in ContextManager (structural trim only in Phase 1)
- Full streaming HTTP in `stream_post_fn_proxy` (stub only; streaming already works via `rust_agent_register_stream_cb`)

## Decisions

### 1. ContentPart Tagged Union (serde tag = "type")

**Choice**: Use `#[serde(tag = "type")]` on `ContentPart` enum, with variants `text`, `reasoning`, `tool_call`, `tool_result`.

**Why**: Different from `AgentRequest` which uses `#[serde(untagged)]` because the action discriminator comes from the C string parameter. ContentPart embeds its own type tag in JSON, matching OpenAI/Anthropic conventions.

**Alternative**: Unified flat struct with optional fields — rejected because tagged union gives exhaustiveness checking and cleaner pattern matching.

### 2. SHA-256 Sliding Window Loop Detection

**Choice**: Hash each tool-call's (name + arguments) and compare against the last 5 rounds. If duplicate found, immediately stop with `LoopDetected`.

**Why**: LLMs can enter infinite loops repeating the same tool call with identical arguments. Signature detection catches this without needing semantic understanding. SHA-256 chosen over cheaper hash (e.g., blake3) because `sha2` crate is audited and no performance requirement for 30-round loops.

**Hard cap**: `MAX_ITERATIONS = 30` as safety net in case the signature detector itself has a bug.

### 3. 16ms Frame-Aligned Throttler (not Timer)

**Choice**: Accumulate SSE deltas in a `Vec<T>` and flush when `elapsed >= 16ms` since last flush.

**Why**: Avoids the overhead of spawning a timer thread. Callers push deltas and check the clock; the throttler returns the batch when the window expires. Aligned to 60fps (1 frame ≈ 16.67ms).

**Alternative**: `tokio::time::interval` — rejected to keep Phase 1 dependency-free (no async runtime yet).

### 4. Condition-Variable Blocking IO Proxy

**Choice**: `post_fn_proxy` pushes a payload via `napi_call_threadsafe_function`, then blocks on `std::condition_variable::wait()`. `DeliverIoToJS` sets the response and calls `notify_one()`.

**Why**: Rust's C ABI interface requires `post_fn` to return `char*` (blocking). The only way to bridge to ArkTS's async `@ohos.net.http` is to block the calling Rust thread until the JS callback completes. Condition variable is the simplest correct primitive.

**Risk**: Deadlock if the JS callback never fires (network timeout). Mitigated by the 30s connect timeout in SystemIoImpl — if the HTTP call fails, the error response is delivered, unblocking the CV.

### 5. BM25 Level 0 with Paragraph Chunking

**Choice**: Split documents on `\n\n` (paragraph boundaries), tokenize by non-alphanumeric chars (min 2 chars), score by term frequency intersection.

**Why**: No external embedding API needed for Phase 1. Paragraph-level chunking gives reasonable relevance for code and markdown documents. Multi-threaded scanning via `std::thread::spawn` with 8-file batches.

**Alternative**: Fixed-size sliding window — rejected because paragraph boundaries preserve semantic units better for code/docs.

### 6. ContextManager Structural Trim (not LLM Summarisation)

**Choice**: Preserve system prompt + last 4 messages; trim tool outputs in older messages to 500 chars + `[truncated]` marker.

**Why**: LLM summarisation requires calling the LLM itself, creating a recursive dependency. Phase 1 establishes the compaction framework; Phase 2+ can add LLM-based summarisation as an upgrade.

### 7. In-Process ripgrep via `ignore` + `grep-*` crates

**Choice**: Link BurntSushi's `ignore`, `grep-regex`, `grep-searcher` crates directly into `agent_core`.

**Why**: HarmonyOS sandbox prohibits `Command::spawn`. The same crates power `ripgrep` and are pure Rust (musl-compatible). `ignore::WalkBuilder` respects `.gitignore`, `.ignore`, and hidden-file rules automatically.

## Risks / Trade-offs

- **[Risk] `memmap2` not available on OHOS musl** → `grep-searcher` can fall back to in-memory reading. If mmap fails at runtime, search will still work (slightly slower).
- **[Risk] Condition variable deadlock in `post_fn_proxy`** → Mitigated by HTTP timeout in SystemIoImpl; the error path always notifies the CV.
- **[Trade-off] No async runtime** → Phase 1 uses blocking threads (`std::thread::spawn`) instead of tokio. Simpler dependency graph, but limits future concurrency. Tokio can be introduced in Phase 3 when HTTP streaming fully implemented.
- **[Trade-off] BM25 only, no vectors** → Search quality is keyword-based. Adequate for code/docs but won't capture semantic similarity. Upgraded in Phase 4 with online embedding API.
