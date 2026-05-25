## ADDED Requirements

### Requirement: SystemIoImpl host IO implementation

The HAR module SHALL contain `SystemIoImpl.ets` providing `httpPost(url, body): string`, `readFile(path): string`, `writeFile(path, content): boolean`, and `fileExists(path): boolean` methods, using `@ohos.net.http` and `@ohos.file.fs` system APIs.

#### Scenario: HTTP POST success
- **WHEN** `httpPost("https://api.example.com", "{...}")` is called
- **THEN** the request is sent via `@ohos.net.http` and the response body is returned as string

#### Scenario: HTTP POST failure
- **WHEN** the HTTP request fails with a network error
- **THEN** a JSON error object `{"status":"error","error":"...","code":...}` is returned

### Requirement: RustAgentBridge Phase 1 methods

`RustAgentBridge.ets` SHALL export `initWithIo(onChunk, onIo): boolean`, `search(dirPath, pattern): string`, and `scanDir(dirPath): string` in addition to Phase 0 methods.

#### Scenario: initWithIo
- **WHEN** bridge.initWithIo(streamCb, ioCb) is called
- **THEN** nativeBridge.initAgentWithIo is invoked with both callbacks
- **AND** returns true on success

#### Scenario: search delegate
- **WHEN** bridge.search("/path", "pattern") is called
- **THEN** nativeBridge.search is invoked and its result returned

## MODIFIED Requirements

### Requirement: 测试 UI 页面

`Index.ets` MUST 提供 6 个测试按钮：Init Agent, Ping Rust, Test Network, Test Stream, Search Files, Scan Dir。页面 MUST 是可滚动的（Scroll + Column）。

#### Scenario: Phase 1 测试按钮
- **WHEN** 用户点击 Search Files 或 Scan Dir
- **THEN** 对应的 NAPI 函数被调用
- **AND** 结果或状态文本显示在页面上
- **AND** hilog 输出 `[TEST]` 前缀日志
