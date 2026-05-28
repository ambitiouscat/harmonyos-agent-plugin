## Context

Phase 7 agent loop 上线后出现 HTTP 400 调用失败。消息格式虽已统一为 OpenAI 标准，但仍有 5 个细节问题阻塞工具调用。同时状态栏 UI 简单的趣味短语 + 工具名布局需要升级为可扩展的动态状态系统。

## Goals / Non-Goals

**Goals:**
- 修复所有 HTTP 400 根因，使 agent loop 可正常调用工具
- 捕获 API 真实错误消息以便后续排查
- 支持 DeepSeek R1 等 thinking 模型的 `reasoning_content` 回传
- 状态栏支持动态插入/移除状态项，自动换行
- 工具颜色可配置，主题持久化

**Non-Goals:**
- 不添加 Claude 原生 API 出站格式支持（留 TODO）
- 不修改后端工具注册逻辑

## Decisions

### 1. ureq 3.x 错误处理：用 `http_status_as_error(false)` 代替 `send_json()`
- **选型**: 创建 Agent with `http_status_as_error(false)` → 手动检查 status → 读取响应体
- **原因**: ureq 3.x `send_json()` 对 4xx 直接返回 `Err(StatusCode(u16))`，丢弃响应体，导致 API 具体错误消息不可见
- **替代方案**: 未采用 `ureq::Error::Status` 匹配（3.3.0 仍为 `StatusCode(u16)` 不含响应体）

### 2. 移除 `tool_choice: "auto"`
- **选型**: 不设置 `tool_choice` 参数
- **原因**: 不设置等同于默认 `"auto"` 行为，且兼容性更广。部分 provider 不认识此参数

### 3. assistant 消息 `reasoning_content` 独立字段
- **选型**: SSE 解析器捕获 `delta.reasoning_content`，`messages_to_api` 输出为独立 `reasoning_content` 键
- **原因**: DeepSeek R1 等 thinking 模型要求后续请求必须回传 `reasoning_content`，否则 400
- **注意**: `content` 和 `reasoning_content` 必须分开传递，不能合并

### 4. 状态栏双层架构
- **StatusBar**（上层动态行）: FlexWrap 布局，趣味短语始终第一位（id='phrase'），工具调用等动态增删
- **InfoBar**（下层固定行）: 模型名(M) / 文件夹(F) / 上下文用量(C) / 内存(R)，紧挨输入框上方
- **状态管理**: `upsertStatusItem(id, label, color)` 插入/更新，`removeStatusItem(id)` 移除，phrase 始终 unshift

### 5. 主题持久化
- **选型**: preferences 持久化 `theme_color` 键，启动时异步读取回填
- **AppStorage**: 保留为同会话快速路径
- **默认值**: light（白色）

## Risks / Trade-offs

- [上下文用量不准] → 按字符数/4 估算 token 数，后续可接入 tokenizer
- [ToolProgressBar 保留但未使用] → 旧组件保留避免破坏其他引用，后续清理
- [preferences 异步读取有延迟] → 白色主题先渲染，冷启动有几帧短暂白色闪烁后再切换
