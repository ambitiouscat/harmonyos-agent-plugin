## abort-chain

### Purpose
AtomicBool-based abort/reset_abort control mechanism enabling the ArkTS host to cleanly terminate in-progress Rust streaming operations (LLM chat, search, RAG scanning) without memory corruption or deadlocks.

### Requirements

- **REQ-AB-001**: A global `AtomicBool ABORT_FLAG` SHALL exist in an independent Rust module (`agent::abort`) with zero inward dependencies
- **REQ-AB-002**: The host SHALL set `ABORT_FLAG` via `bridge.call("abort", "{}")` and clear it via `bridge.call("reset_abort", "{}")`
- **REQ-AB-003**: `LoopEngine::can_continue()` SHALL return `false` when `ABORT_FLAG` is set
- **REQ-AB-004**: `chat_completion_ureq()` SHALL check `ABORT_FLAG.load(Ordering::Relaxed)` between each SSE event iteration and return `Err("aborted")` if set
- **REQ-AB-005**: ArkTS side SHALL call `bridge.resetAbort()` before each new `chat` call to clear stale abort state

### Implementation

- `agent/abort.rs`: `pub static ABORT_FLAG: AtomicBool = AtomicBool::new(false)`
- `json_router.rs`: `"abort"` / `"reset_abort"` actions
- `agent/chat.rs`: `if ABORT_FLAG.load(Ordering::Relaxed) { return Err("aborted".into()); }` in SSE event loop
- `ChatView.ets`: `handleStop()` → `bridge.abort()`, `handleSend()` → `bridge.resetAbort()`
