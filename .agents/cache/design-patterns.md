---
generated_at: 2026-05-27
patterns:
  - JSON Action Router
  - NAPI Thread-Safe Bridge
  - SSE Streaming with Interleaved Timers
  - ReAct Agent Loop
  - Singleton Bridge Pattern
  - Platform Feature Gating
  - 3-Tier Provider Loading
  - @ObservedV2 State Management
  - Sandboxed File Operations
  - Sliding-Window Loop Detection
---

# Design Patterns & Conventions

## 1. JSON Action Router (Command Pattern)

**Where**: `rust_core/agent_core/src/json_router.rs`

All ArkTS→Rust communication goes through a single dispatch function:
- `rust_agent_call(action: &str, json_args: &str) -> *mut c_char`
- ~20 actions: ping, chat, configure, abort, init_session, session CRUD, vfs_write_file, test_stream
- Each action maps to a handler function
- Unknown actions return JSON error response

**Why**: Single chokepoint for logging, error handling, and future middleware. Avoids N×M FFI function explosion.

## 2. NAPI Thread-Safe Bridge (Bridge Pattern)

**Where**: `native_bridge.cpp` + `RustAgentBridge.ets`

- C++ `napi_threadsafe_function` allows Rust worker threads to call back into ArkTS main thread
- Two callback types: `post_fn` (final result) and `stream_post_fn` (SSE chunks)
- `SystemCallbacks` struct passed at init time

**Why**: Rust HTTP calls happen on worker threads; ArkTS UI updates must happen on main thread. Thread-safe functions bridge this gap without polling.

## 3. SSE Streaming with Interleaved Timers (Observer Pattern)

**Where**: `SseStreamController.ets`

- Parses `data: ...` SSE events into text chunks
- Uses interleaved timers (~20ms delay) between chunks
- Purpose: yields control to ArkUI render loop so UI doesn't freeze during streaming
- Supports cancellation via AbortController

**Why**: ArkUI's rendering is synchronous per frame. Without yielding, rapid SSE chunks would block the UI thread.

## 4. ReAct Agent Loop (Chain of Responsibility)

**Where**: `rust_core/agent_core/src/agent/loop_engine.rs`

- Think → Act → Observe cycle
- Each iteration: LLM decides to respond or call a tool
- Tool calls return results, fed back into context for next iteration
- SHA-256 fingerprinting detects repeated states (sliding window of 5)
- Hard limit: 30 iterations
- User abort via global `AtomicBool`

**Why**: Standard ReAct pattern allows the agent to use tools iteratively. Loop detection prevents infinite tool-calling cycles.

## 5. Singleton Bridge Pattern

**Where**: `RustAgentBridge.ets`

- Single `RustAgentBridge` instance with lazy initialization
- Provides: `init()`, `call()`, `search()`, `scanDir()`, `initSession()`, `abort()`, `resetAbort()`
- `callWithRetry()` with exponential backoff
- Offline detection tracking

**Why**: NAPI module initialization is expensive; single instance ensures consistent state and avoids double-init.

## 6. Platform Feature Gating (Strategy Pattern)

**Where**: `rust_core/agent_core/Cargo.toml` + `agent/wasm.rs` + `agent/node.rs`

- `#[cfg(feature = "wasm")]` gates WASM-specific code
- `#[cfg(feature = "node")]` gates Node.js NAPI code
- Default (no feature): HarmonyOS staticlib with FFI callbacks
- Each platform has its own adapter implementing the same abstract interface

**Why**: Single codebase, multiple targets. Feature flags ensure platform-specific code doesn't leak.

## 7. 3-Tier Provider Loading (Fallback Pattern)

**Where**: `provider/ProviderLoader.ets`

1. **Embedded**: Rawfile `providers.json` (always available)
2. **Cached**: Sandbox file (7-day TTL)
3. **Remote**: HTTP fetch from `models.dev`

Priority: remote > cached > embedded. Falls back gracefully.

**Why**: Users get updated provider lists without app updates, but the app never breaks offline.

## 8. @ObservedV2 State Management (Observer Pattern)

**Where**: `ui/models/ArkMessage.ets` + `ArkContentPart.ets`

- `@ObservedV2` decorator on model classes
- `@Trace` on mutable fields (text, collapsed, parts array)
- ArkUI auto-re-renders when `@Trace` fields change
- `applyDelta()` in ChatView mutates `@Trace` fields to trigger incremental UI updates

**Why**: Fine-grained reactivity avoids full-list re-renders during streaming; only changed text nodes update.

## 9. Sandboxed File Operations (Proxy Pattern)

**Where**: `tools/file.rs` + `sandbox/validate.rs`

- All file paths canonicalized before access
- Path traversal attacks blocked by checking canonical prefix
- File operations go through `SystemIoImpl.ets` host callbacks (not direct Rust fs access)
- Sandbox validation: network test (TCP to 8.8.8.8:53), file write test, stream simulator

**Why**: Running untrusted model-generated tool calls requires safety boundaries.

## 10. Sliding-Window Loop Detection

**Where**: `agent/loop_engine.rs`

- SHA-256 hash of (tool_name + arguments) for each tool call
- Sliding window of last 5 hashes
- If current hash matches any in window → loop detected
- Different arguments to same tool are allowed (hash differs)

**Why**: Prevents infinite tool-calling loops while allowing legitimate repeated tool use with different parameters.

## Coding Conventions

### ArkTS
- `@ObservedV2` + `@Trace` for reactive state
- PascalCase for components, camelCase for methods
- Components in dedicated subdirectories with `index.ets` re-exports
- `@Builder` for reusable UI fragments
- NAPI calls wrapped in try/catch with error logging

### Rust
- Standard Rust 2021 conventions
- `#[cfg(test)] mod tests` at bottom of each source file
- `pub mod` re-exports in `lib.rs`
- `#[no_mangle] pub extern "C"` for FFI exports
- `serde::Serialize/Deserialize` on all message types
- `#[serde(tag = "type")]` for ContentPart enum variants

### C++
- NAPI value creation/destruction paired
- `napi_threadsafe_function` with proper ref counting
- Native logging via `OH_LOG_*` macros
- ABI-specific preprocessor guards for library paths

## Explosion Radius

Key modules with high impact radius:
1. `ffi.rs` — All C ABI exports; breaking changes affect all platforms
2. `types/message.rs` — Message format shared across all modules; schema changes cascade
3. `json_router.rs` — Single dispatch point; action name changes break ArkTS callers
4. `RustAgentBridge.ets` — All UI code depends on this singleton
5. `native_bridge.cpp` — Thread-safety bugs crash entire app

## Developer Templates

### Adding a new Rust action:
1. Add handler in `json_router.rs`
2. If new type needed, add to `types/message.rs`
3. Expose via `ffi.rs` if new C function needed
4. Add NAPI wrapper in `native_bridge.cpp`
5. Add ArkTS method in `RustAgentBridge.ets`

### Adding a new UI component:
1. Create `ComponentName.ets` in appropriate `ui/` subdirectory
2. Add `index.ets` re-export
3. Import and use in `ChatView.ets` or parent component
4. If stateful, use `@ObservedV2` + `@Trace` pattern

