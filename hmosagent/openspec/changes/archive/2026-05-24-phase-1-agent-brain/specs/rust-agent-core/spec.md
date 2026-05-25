## ADDED Requirements

### Requirement: Polymorphic ContentPart message model

The Message struct SHALL support a `parts: Vec<ContentPart>` field, where ContentPart is a tagged union with variants `text`, `reasoning`, `tool_call`, `tool_result`. Each variant SHALL carry type-specific fields serialized with `#[serde(tag = "type")]`.

#### Scenario: Text message round-trip
- **WHEN** a Message with `ContentPart::Text { text: "Hello" }` is serialized to JSON and back
- **THEN** the deserialized message has role "assistant" and one text part

#### Scenario: Tool call serialization
- **WHEN** a `ContentPart::ToolCall` is serialized
- **THEN** the JSON contains `"type":"tool_call"` and the tool id, name, and arguments

### Requirement: ReAct loop engine

The system SHALL provide `AgentLoop` with SHA-256 sliding-window loop detection (window size 5) and a hard iteration cap of `MAX_ITERATIONS = 30`.

#### Scenario: Duplicate tool call detected
- **WHEN** `record_tool_call("search", r#"{"q":"hello"}"#)` is called twice with identical arguments
- **THEN** the second call returns false (loop detected)

#### Scenario: Different arguments allowed
- **WHEN** three calls with different arguments are made
- **THEN** all three return true

#### Scenario: Hard limit reached
- **WHEN** 30 iterations are performed
- **THEN** `can_continue()` returns false and `finish(false)` returns `MaxIterationsReached`

### Requirement: SSE pipeline with frame throttling

The system SHALL provide `SseMerger` for incremental SSE data line merging and `FrameThrottler<T>` for 16ms-window batched output.

#### Scenario: Complete SSE data line
- **WHEN** `feed("data: {\"delta\":\"hello\"}")` is called
- **THEN** the complete JSON object is returned

#### Scenario: Partial then complete
- **WHEN** `feed("data: {\"x\":\"")` returns None, then `feed("data: [DONE]")` is called
- **THEN** the accumulated partial content is flushed as residual

#### Scenario: Frame throttler accumulates
- **WHEN** two items are pushed within 16ms
- **THEN** `push()` returns None for both
- **AND** `flush()` returns both items

### Requirement: Context window manager

The system SHALL provide `ContextManager` with configurable token budget. Compaction SHALL trigger when estimated tokens exceed 80% of budget.

#### Scenario: Compaction triggered
- **WHEN** messages total >80% of token budget
- **THEN** `needs_compaction()` returns true

#### Scenario: System prompt preserved
- **WHEN** compaction runs
- **THEN** messages with role "system" are never modified

#### Scenario: Old tool output trimmed
- **WHEN** a ToolResult in an old message (not in last 4) exceeds 500 chars
- **THEN** it is truncated to 500 chars + "…[truncated]" marker

## MODIFIED Requirements

### Requirement: C FFI 接口暴露

Rust 核心 MUST 通过 `extern "C"` 暴露至少以下 8 个符号：

| Symbol | Signature | Purpose |
|--------|-----------|---------|
| `rust_agent_init` | `fn(SystemCallbacks) -> bool` | 注入宿主 IO 能力 |
| `rust_agent_call` | `fn(*const c_char, *const c_char) -> *mut c_char` | 统一 JSON 消息路由 |
| `rust_agent_free_str` | `fn(*mut c_char)` | 释放返回字符串 |
| `rust_agent_register_stream_cb` | `fn(extern "C" fn(...))` | 注册流式回调 |
| `rust_agent_search` | `fn(*const c_char, *const c_char) -> *mut c_char` | 进程内文件搜索 |
| `rust_agent_scan_dir` | `fn(*const c_char) -> *mut c_char` | RAG 目录扫描 |
| `test_network` | `fn() -> bool` | 沙箱网络验证 |
| `test_file` | `fn(*const c_char) -> bool` | 沙箱文件写验证 |

#### Scenario: 宿主初始化 Rust 核心
- **WHEN** 宿主调用 `rust_agent_init(callbacks)` 且 callbacks 为有效 `SystemCallbacks` 结构体
- **THEN** 返回 true，全局回调表被 OnceLock 存储
- **AND** 后续调用返回 false（OnceLock 不可重复设置）

#### Scenario: 搜索调用
- **WHEN** 宿主调用 `rust_agent_search("/path", "pattern")`
- **THEN** 返回 JSON 格式的搜索结果字符串
- **AND** 宿主必须调用 `rust_agent_free_str` 释放该指针

#### Scenario: RAG 扫描调用
- **WHEN** 宿主调用 `rust_agent_scan_dir("/path")`
- **THEN** 返回 JSON `{"status":"ok","chunks_indexed":N}`
- **AND** 宿主必须调用 `rust_agent_free_str` 释放该指针
