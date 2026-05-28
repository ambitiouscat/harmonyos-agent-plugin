## Why

Phase 7 agent loop 上线后发现 HTTP 400 无法调用工具，根因排查经历了 5 轮修复才完全解决。同时状态栏 UI 需要重构以支持动态状态展示和持久化主题配置。

## What Changes

- **Bug 修复**: 修复 agent loop 中 LLM API 调用的 HTTP 400 错误（5 轮迭代）
  - 死代码错误处理：ureq 3.x `send_json()` 丢弃 4xx 响应体
  - 移除 `tool_choice: "auto"` 提升 API 兼容性
  - 移除 assistant 纯 tool_calls 时的 `content: null`
  - 防御性 `arguments` 规范化（null → `{}`）
  - `reasoning_content` SSE 捕获与回传（DeepSeek R1 等 thinking 模型强制要求）
- **TODO 记录**: Claude 原生 API 出站格式待支持，Trilium 笔记记录
- **状态栏重构**: 新增 `StatusBar`（动态状态项，FlexWrap 自动换行）和 `InfoBar`（模型名/文件夹/上下文用量固定展示）
- **工具颜色可配置**: 每个工具 4 主题独立配色，写入 `themes.json`
- **默认白色主题 + 持久化**: 启动默认 light 主题，选择通过 preferences 持久化存储

## Capabilities

### New Capabilities
- `tool-progress-status-bar`: 动态状态栏组件，支持趣味短语、工具调用等状态项自动插入/移除，固定信息栏显示模型、文件夹、上下文用量
- `theme-persistence`: 主题选择持久化到 preferences，重启不丢失

### Modified Capabilities
- `agent-loop`: HTTP 错误处理改造，`reasoning_content` 回传，移除 `tool_choice: auto`、`content: null`
- `tool-colors`: 工具颜色从 themes.json 配置，支持每个工具独立配色

## Impact

- Rust: `rust_core/agent_core/src/agent/loop_engine.rs`（`llm_api_call`, `messages_to_api`, `LlmResponse`, `normalize_args`）
- ArkTS: `ChatView.ets`（状态管理 + StatusBar/InfoBar 集成）
- ArkTS 新增: `StatusBar.ets`, `InfoBar.ets`
- ArkTS 修改: `ThemeLoader.ets`（持久化 + 默认主题）, `ToolProgressBar.ets`（字体/颜色）, `themes.json`（tool_colors）, `ProviderLoader.ets`（ModelLimit 接口）
