## ADDED Requirements

### Requirement: UI component library

The HAR module SHALL export 10 reusable UI components under `src/main/ets/ui/`: ArkMessage, ArkContentPart, MarkdownParser, MarkdownView, ThinkingPanel, ToolCard, MessageBubble, InputBar, ChatView, SettingsPage.

#### Scenario: HAR exports
- **WHEN** entry module imports from `hmos_agent_core`
- **THEN** `ChatView`, `SettingsPage`, `ArkMessage`, `ArkContentPart` are available

### Requirement: SettingsPage in HAR

`SettingsPage.ets` SHALL reside in the HAR module at `pages/SettingsPage.ets` and SHALL be exported via `Index.ets`.

## MODIFIED Requirements

### Requirement: ArkTS 桥接类

`RustAgentBridge.ets` MUST 封装 NAPI 调用，提供 `init()`, `initWithIo()`, `call()`, `testNetwork()`, `search()`, `scanDir()` 六个公共方法。流式回调接收 ContentPart JSON 格式的 data 参数。

### Requirement: 测试 UI 页面

`Index.ets` SHALL render a chat-style interface with `@State messages: ArkMessage[]`, a stream delta handler, an input text field, and a send/stream-test button. The Phase 0/1 6-button debug console is removed.

#### Scenario: Stream test renders deltas
- **WHEN** user taps "Test Stream"
- **THEN** structured JSON deltas are received, parsed, and rendered as expanding text in a chat bubble style
