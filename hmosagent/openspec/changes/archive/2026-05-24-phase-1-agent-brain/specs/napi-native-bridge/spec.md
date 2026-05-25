## ADDED Requirements

### Requirement: Blocking IO proxy via condition variable

The bridge SHALL implement `post_fn_proxy`, `stream_post_fn_proxy`, and `free_str_fn_proxy` as C-compatible static functions conforming to the `SystemCallbacks` function pointer types. `post_fn_proxy` SHALL block the calling thread via `std::condition_variable::wait()` until the ArkTS IO callback delivers a response.

#### Scenario: post_fn_proxy with valid IO tsfn
- **WHEN** `post_fn_proxy(url, body)` is called and `g_io_tsfn` is initialized
- **THEN** the payload is dispatched via `napi_call_threadsafe_function(g_io_tsfn, ..., napi_tsfn_blocking)`
- **AND** the calling thread blocks on `g_io_cv`
- **AND** when ArkTS responds, the thread wakes and returns a `malloc`-allocated response string

#### Scenario: post_fn_proxy without IO tsfn
- **WHEN** `post_fn_proxy` is called but `g_io_tsfn` is null
- **THEN** returns `{"status":"error","error":"IO not initialized"}` immediately without blocking

### Requirement: initAgentWithIo dual tsfn

The bridge SHALL export `initAgentWithIo(streamCb, ioCb)` which creates both `g_stream_tsfn` and `g_io_tsfn`, registers `OnChunkBridge`, and calls `rust_agent_init` with real C proxy functions.

#### Scenario: initAgentWithIo wires SystemCallbacks
- **WHEN** ArkTS calls `nativeBridge.initAgentWithIo(streamCb, ioCb)`
- **THEN** `SystemCallbacks.post_fn` = `post_fn_proxy` (not nullptr)
- **AND** `SystemCallbacks.stream_post_fn` = `stream_post_fn_proxy`
- **AND** `SystemCallbacks.free_str_fn` = `free_str_fn_proxy`
- **AND** `rust_agent_init` returns true on first call

### Requirement: search and scanDir NAPI exports

The bridge SHALL export `search(dir, pattern)` and `scanDir(dir)` NAPI functions that delegate to `rust_agent_search` and `rust_agent_scan_dir` respectively, logging results with a `[TEST]` prefix.

#### Scenario: search NAPI call
- **WHEN** ArkTS calls `nativeBridge.search("/sandbox", "error")`
- **THEN** the result JSON string is returned
- **AND** a `[TEST]` prefixed hilog entry is emitted

### Requirement: Unified test logging prefix

All NAPI test functions SHALL log with a `[TEST]` prefix for unified log filtering via `hilog | grep "\[TEST\]"`.

#### Scenario: Log filter by prefix
- **WHEN** any test button (agentCall, testNetwork, search, scanDir) is clicked
- **THEN** the corresponding hilog entry begins with `[TEST]`

## MODIFIED Requirements

### Requirement: rust_agent_init 调用

`initAgent(jsCallback)` NAPI 函数 SHALL 创建 `napi_threadsafe_function` 并调用 `rust_agent_init`。`initAgentWithIo(streamCb, ioCb)` SHALL additionally create an IO tsfn and wire real C proxy functions into `SystemCallbacks`.

#### Scenario: 首次初始化 (Phase 0 compat)
- **WHEN** ArkTS 调用 `nativeBridge.initAgent(callback)` 且首次调用
- **THEN** `rust_agent_init` 返回 true
- **AND** napi_threadsafe_function 创建成功
- **AND** SystemCallbacks wired with real proxies (not nullptr) in Phase 1 mode

#### Scenario: initAgentWithIo (Phase 1 full)
- **WHEN** ArkTS 调用 `nativeBridge.initAgentWithIo(streamCb, ioCb)` 且首次调用
- **THEN** two tsfn handles are created
- **AND** SystemCallbacks.post_fn, .stream_post_fn, .free_str_fn are all non-null
- **AND** `rust_agent_init` returns true

### Requirement: 模块注册

HAR 模块 MUST 通过 `napi_module_register` 注册名为 `"native_bridge"` 的模块，导出 7 个函数：`initAgent`, `initAgentWithIo`, `agentCall`, `testNetwork`, `testFile`, `search`, `scanDir`。

#### Scenario: ArkTS 导入
- **WHEN** ArkTS 编写 `import nativeBridge from 'libnative_bridge.so'`
- **THEN** 可调用所有 7 个导出函数
