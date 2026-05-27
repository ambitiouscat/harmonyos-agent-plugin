# Proposal: Phase 6 — 跨平台生态与未完残片彻底贯通

## Why

Phase 0-5 完成了 Rust 核心引擎、ArkUI V2 渲染、Godot 解耦集成、多平台打包验证和趣味词系统。但 13 个功能残片遗留未实施，涵盖会话管理、技能注册、命令补全、高阶工具、主题系统、SSE 重连等领域。Phase 6 以跨平台 Rust 核心内聚为原则，将这些残片统一实现，交付工业级无断层智能体方案。

## What

### Component A: Rust Core 升级
1. **SessionManager** (agent/session.rs) — JSON 持久化多会话 CRUD，filesDir 宿主注入，5 FFI actions
2. **SkillsRegistry** (agent/skills.rs) — 斜杠命令注册引擎，prefix/fuzzy match 检索
3. **高阶工具集** (tools/) — FileTools (沙箱文件), WebTools (post_fn 代理), McpBridge (同步 JSON-RPC), SubAgentRunner (多线程)
4. **SSE 重连** (chat.rs) — ReconnectConfig 指数退避 retry loop，跨重连去重合并

### Component B: HAR UI 抛光
5. **ConversationList** — 左滑抽屉，多会话切换，FFI CRUD 绑定
6. **CommandPalette** — / 触发悬浮补全，FFI 动态拉取命令列表
7. **主题系统** — ThemeLoader + 4 套预设 (Dark/Light/Sepia/Solarized), 循环切换
8. **打字机动效** — 流式末段闪烁光标，block 级 OPACITY 渐显
9. **Session 延迟入库** — 空 session 不写盘，首条消息触发 create_session
10. **偏好持久化修复** — AppStorage 冷启动从 Preferences KV 回读
11. **设置浮层化** — SettingsPage 全屏 overlay，不销毁消息 Scroll

### Component C: 测试与归档
12. Session/Skills/Tools 三组 Rust 测试 (12 new tests)
13. 42 SVG 图标替换全部 emoji 按钮
14. 7 编译脚本 + CLAUDE.md
15. 三端编译验证 (HAR + WASM + Node)

## Scope

- **In**: session.rs, skills.rs, tools/*, chat.rs retry, ConversationList.ets, CommandPalette.ets, ThemeLoader.ets, themes.json, MarkdownView cursor, ChatView overlay/session/pagination, 42 icons, 7 build scripts
- **Out**: 打字机逐字符动画 (ArkUI V2 setInterval 限制，回退为光标方案)
- **Platforms**: HarmonyOS HAP, WASM (wasm32-unknown-unknown), Node.js (napi-rs)

## Impact

- **New specs**: session-manager, skills-registry, conversation-drawer, command-palette, theme-system, advanced-tools, sse-reconnect
- **New code**: 7 Rust files (session, skills, tools×4, tools/mod), 3 ETS files (ConversationList, CommandPalette, ThemeLoader), 42 SVG icons, 2 JSON configs (themes.json, app_config.json), 5 build scripts
- **Modified code**: ChatView.ets (~630 lines), MarkdownView.ets, InputBar.ets, RustAgentBridge.ets, MessageBubble.ets, build-profile.json5, Cargo.toml, chat.rs, json_router.rs, ffi.rs, pipeline.rs, wasm.rs, node.rs, lib.rs, mod.rs, types/message.rs, env-setup.sh, cross-compile.sh
- **Risk**: 低 — Phase 5 已预留工具集+会话管理给 Phase 6；Rust SSE retry 去重有边界情况但不影响正确性
- **Git**: 2f4d227, 96 files, +5836/-263
