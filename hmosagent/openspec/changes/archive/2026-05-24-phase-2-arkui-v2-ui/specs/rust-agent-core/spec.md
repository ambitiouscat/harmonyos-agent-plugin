## ADDED Requirements

### Requirement: configure action route

`json_router::dispatch` SHALL handle the `"configure"` action, parsing the JSON args and storing them in a global `RwLock<Value>`. Valid JSON SHALL return `{"status":"ok"}`. Invalid JSON SHALL return `{"status":"error"}`.

#### Scenario: Configure success
- **WHEN** action="configure", args='{"api_key":"sk-abc"}'
- **THEN** config stored in AGENT_CONFIG
- **AND** response contains "Configuration stored"

#### Scenario: Configure invalid JSON
- **WHEN** action="configure", args='not json'
- **THEN** response contains "Invalid config JSON"

## MODIFIED Requirements

### Requirement: C FFI 接口暴露

Rust 核心 MUST 通过 `extern "C"` 暴露至少以下 8 个符号。流式回调的 content 格式 SHALL 为 ContentPart JSON 字符串。

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
- **THEN** 返回 true

#### Scenario: 结构化流式回调
- **WHEN** Rust 通过 stream callback 推送数据
- **THEN** 数据格式为 ContentPart JSON（如 `{"type":"text","text":"H"}`）
