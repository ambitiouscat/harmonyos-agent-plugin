# Tasks: Phase 6 — 跨平台生态与未完残片彻底贯通

## Gate Criteria

1. `cargo test` 47/47 pass
2. HAR + HAP BUILD SUCCESSFUL
3. WASM + Node 编译 OK
4. 真机: 抽屉/命令补全/主题切换/session CRUD/offline banner

---

## Task 6.1: SessionManager ✅

- [x] `agent/session.rs`: JSON 持久化, 5 FFI, 4 tests
- [x] `json_router.rs`: session 路由, 直接 JSON 解析
- [x] `ffi.rs`: `rust_agent_init_session`
- [x] ffi_integration test 更新

## Task 6.2: SkillsRegistry ✅

- [x] `agent/skills.rs`: 5 commands, fuzzy match, 5 tests
- [x] `json_router.rs`: `get_registered_skills` 路由
- [x] LazyLock 全局单例

## Task 6.3: ConversationList ✅

- [x] `ConversationList.ets`: 80% 面板, ✕/+ 按钮, backdrop 关闭
- [x] ChatView 集成: if showDrawer 条件渲染
- [x] asymmetric transition: translate(-2000)
- [x] 会话标题居中

## Task 6.4: CommandPalette ✅

- [x] `CommandPalette.ets` + InputBar / 拦截
- [x] filteredSkills() 模糊匹配
- [x] ChatView loadSkills + skillList

## Task 6.5: 主题 + 打字机 ✅

- [x] `ThemeLoader.ets` + `themes.json` 4 套预设
- [x] 全组件主题绑定
- [x] 闪烁光标 + OPACITY 渐显

## Task 6.6: 高阶工具 ✅

- [x] `tools/file.rs` (沙箱, 3 tests)
- [x] `tools/web.rs` (post_fn 代理)
- [x] `tools/mcp.rs` (JSON-RPC, wasm32 隔离)
- [x] `tools/subagent.rs` (threaded, 2 tests, wasm32 隔离)

## Task 6.7: SSE 重连 ✅

- [x] `chat.rs`: retry loop + 去重
- [x] `RustAgentBridge.ets`: callWithRetry + offline banner

## Task 6.8: 测试归档 ✅

- [x] 47/47 tests, 三端编译, 真机部署

## UI 抛光 (超计划) ✅

- [x] 42 SVG 图标, 滚动优化, 历史分页, session 延迟入库, 偏好持久化, 趣味词随机, 设置浮层

## Verification ✅

- [x] cargo test 47/47
- [x] HAR + HAP BUILD SUCCESSFUL
- [x] WASM + Node 编译 OK
- [x] 真机部署验证通过
- [x] 7 编译脚本全部通过
- [x] Git: 2f4d227, 96 files, +5836/-263
