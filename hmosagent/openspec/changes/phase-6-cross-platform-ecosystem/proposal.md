## Why

Phase 0-5 完成后遗留 13 个功能残片：会话管理、技能注册、命令补全、高阶工具、主题系统、SSE 重连、离线降级。需以跨平台 Rust 核心内聚为原则统一实现，交付工业级无断层智能体方案。

## What Changes

- **Rust Core**: 新增 SessionManager（5 FFI actions）、SkillsRegistry（5 slash commands）、高阶工具集（File/Web/MCP/SubAgent）、SSE 重连 retry loop
- **ArkUI V2**: 新增 ConversationList 左滑抽屉、CommandPalette / 补全、4 套主题系统、打字机光标、SettingsPage 浮层化
- **UI/UX**: 42 SVG 图标替换全部 emoji、Session 延迟入库、历史分页、趣味词随机+持久化修复、自动滚底
- **工程化**: 7 编译脚本、CLAUDE.md、SDK 6.0.2(22) 升级、三端编译验证

## Capabilities

### New Capabilities

- `session-manager`: 多会话 JSON 持久化 CRUD，filesDir 注入，延迟入库
- `skills-registry`: 斜杠命令注册引擎，fuzzy match，FFI 动态拉取
- `conversation-drawer`: 左滑抽屉，transition 动画，FFI session CRUD 绑定
- `command-palette`: / 触发悬浮补全，FFI get_registered_skills
- `theme-system`: ThemeLoader + 4 套预设 (Dark/Light/Sepia/Solarized)，可编辑 JSON
- `advanced-tools`: FileTools (沙箱), WebTools (post_fn), McpBridge (JSON-RPC), SubAgentRunner (threaded)
- `sse-reconnect`: ReconnectConfig 指数退避 retry，跨重连去重合并

### Modified Capabilities

<!-- None - all capabilities are new in Phase 6 -->

## Impact

- **New**: 14 files (3 Rust modules + 4 tools + 3 ETS components + 2 JSON configs + 42 SVG icons + 5 build scripts)
- **Modified**: 21 files (ChatView, MarkdownView, InputBar, MessageBubble, RustAgentBridge, build-profile.json5, Cargo.toml, chat.rs, json_router.rs, ffi.rs, pipeline.rs, wasm.rs, node.rs, lib.rs, mod.rs, types/message.rs, env-setup.sh, cross-compile.sh, index.ets×2, ffi_integration.rs)
- **Git**: 2f4d227, 96 files, +5836/-263
