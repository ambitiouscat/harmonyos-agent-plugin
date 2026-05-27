# Tasks: Phase 6 — 跨平台生态与未完残片彻底贯通

## Gate Criteria

1. `cargo test` 47/47 pass (40 unit + 7 integration)
2. HAR BUILD SUCCESSFUL (0 errors)
3. HAP BUILD SUCCESSFUL + 真机部署验证
4. WASM `cargo build --target wasm32-unknown-unknown --features wasm` OK
5. Node `cargo build --release --features node` OK
6. 真机: 抽屉打开/关闭, / 命令补全, 主题切换, offline banner, session CRUD

---

## Task 6.1: Rust Core SessionManager ✅

- [x] 创建 `agent/session.rs` (SessionManager, SessionMeta, Session structs)
- [x] JSON 文件持久化: {filesDir}/sessions/{id}.json + index.json
- [x] 5 FFI actions: create_session, list_sessions, load_session, delete_session, save_session
- [x] json_router 路由注册 (直接解析 JSON, 避免 untagged enum 单字段歧义)
- [x] ffi.rs 新增 `rust_agent_init_session(files_dir)`
- [x] 4 unit tests: create_and_list, load_and_delete, save_session, multiple_sessions
- [x] ffi_integration test 更新 (stub → real SessionManager)

## Task 6.2: Rust Core SkillsRegistry ✅

- [x] 创建 `agent/skills.rs` (SkillsRegistry, SkillDef, SkillParam)
- [x] 5 内置命令: /search, /scan, /goal, /file, /session
- [x] get_all(), match_prefix(), fuzzy_match() 检索
- [x] LazyLock<RwLock<SkillsRegistry>> 全局单例
- [x] json_router 新增 `get_registered_skills` 路由
- [x] 5 unit tests: builtin_skills, prefix_match, fuzzy_match, fuzzy_match_partial, fuzzy_match_finds_multiple

## Task 6.3: ConversationList 滑动抽屉 ✅

- [x] 创建 `ui/conversation/ConversationList.ets` (80% 宽暗色面板)
- [x] 创建 `ui/conversation/index.ets` barrel export
- [x] ChatView Stack 集成: if (showDrawer) 条件渲染
- [x] ✕ 关闭按钮 + backdrop 透明点击关闭
- [x] asymmetric transition: translate(-2000) 入场 220ms FastOutSlowIn, 退场 180ms FastOutLinearIn
- [x] ChatView 新增 loadSessions/selectSession/deleteSession/createSession
- [x] 会话标题居中显示 (activeSessionTitle)

## Task 6.4: CommandPalette 斜杠命令补全 ✅

- [x] 创建 `ui/input/CommandPalette.ets` (悬浮列表)
- [x] InputBar.ets / 拦截: onChange → showPalette=true
- [x] filteredSkills() 模糊匹配 + 去重 + 最多 6 条
- [x] 点击补全: handleSelectCommand → inputText=cmd+' '
- [x] ChatView 新增 loadSkills() + skillList, 传入 InputBar
- [x] 更新 `ui/input/index.ets` 导出 CommandPalette

## Task 6.5: 主题系统 + 打字机动效 ✅

- [x] 创建 `ThemeLoader.ets` 单例 (rawfile themes.json 解析)
- [x] 创建 `themes.json` 4 套预设: Dark/Light/Sepia/Solarized
- [x] ChatView: toggleTheme() → cycleNext(), 顶部栏 icon_refresh 按钮
- [x] ChatView/MarkdownView/InputBar 颜色从 ThemeConfig 动态读取
- [x] MarkdownView: AppStorage 回退方案 (V2 不支持 @StorageLink)
- [x] MarkdownView: 闪烁光标 | (500ms 定时器, isStreaming 时末段显示)
- [x] block 级 OPACITY transition 200ms 渐显保留
- [x] 更新 `ui/common/index.ets` 导出 ThemeLoader/ThemeConfig

## Task 6.6: 高阶工具集 ✅

- [x] 创建 `tools/mod.rs` 模块声明
- [x] `tools/file.rs` (FileTools): 沙箱路径 resolve, read/write/list, 3 tests
- [x] `tools/web.rs` (WebTools): post_fn 代理, DDG HTML 解析, c_char 兼容
- [x] `tools/mcp.rs` (McpBridge): JSON-RPC over stdio, tools/list + tools/call
- [x] `tools/subagent.rs` (SubAgentRunner): OS thread spawn, 2 tests
- [x] tools/mcp + subagent #[cfg(not(target_arch = "wasm32"))]
- [x] lib.rs 注册 `pub mod tools`

## Task 6.7: SSE 重连 + 离线降级 ✅

- [x] chat.rs: chat_completion_ureq_with_retry (ReconnectConfig 参数)
- [x] retry loop: HTTP POST → SSE parse → 去重合并 → 成功/耗尽
- [x] dedup_delta() + find_overlap() 跨重连内容去重
- [x] reconnect delta type → applyDelta 识别, 状态栏 "Reconnecting…"
- [x] RustAgentBridge.ets: callWithRetry (3 次指数退避), onOfflineChange, resetOffline
- [x] ChatView: isOffline banner (icon_warning + retry 按钮)
- [x] pipeline.rs: ReconnectConfig { max_retries:3, base_delay_ms:500, max_delay_ms:5000 }

## Task 6.8: 测试归档 ✅

- [x] session 4 tests, skills 5 tests, tools 5 tests (file×3 + subagent×2)
- [x] 全量 cargo test 47/47 pass (40 unit + 7 integration)
- [x] WASM 编译: cargo build --target wasm32-unknown-unknown --features wasm ✅
- [x] Node 编译: cargo build --release --features node ✅
- [x] HAR+HAP 编译验证 (0 errors)

---

## Component D: UI/UX 抛光 (超计划)

- [x] 42 SVG 图标: 替换全部 emoji 按钮 (menu/settings/send/stop/add/close/delete/copy/retry/warning...)
- [x] 滚动优化: Scroll+Scroller 替代 List, scrollToBottom 250ms 节流
- [x] 历史分页: app_config.json message_page_size=30, loadMore 翻页
- [x] Session 延迟入库: 启动/新建仅本地 ID, 首条消息才 create_session
- [x] 趣味词持久化: AppStorage 冷启动从 Preferences KV 回读
- [x] 趣味词随机: 每次随机不等同, 间隔 5~10s 随机
- [x] 动态间距: statusVisible 时 spacer 8px, 否则 4px
- [x] 设置浮层化: SettingsPage 全屏 overlay, 不销毁 Scroll

## Component E: 工程化 (超计划)

- [x] 7 编译脚本: test-rust.sh, build-har.sh, build-app.sh, build-node.sh, build-wasm.sh, cross-compile.sh, build-all.sh
- [x] CLAUDE.md (编译约束 + 技术约束 + 路径索引)
- [x] SDK 升级: compatibleSdkVersion 5.0.0(12) → 6.0.2(22)
- [x] Cargo.toml: ureq 条件依赖 (非 wasm32), getrandom js feature
- [x] wasm.rs/node.rs: unsafe fn → safe wrapper (Rust edition 2024)

---

## Verification ✅

- [x] cargo test 47/47 pass (40 unit + 7 integration)
- [x] HAR BUILD SUCCESSFUL (0 errors, ~9s)
- [x] HAP BUILD SUCCESSFUL (0 errors, ~12s, signed)
- [x] 真机部署 + UI 验证通过
- [x] WASM cargo build --target wasm32-unknown-unknown ✅
- [x] Node cargo build --release --features node ✅ (3.1MB .dll)
- [x] 7 编译脚本全部通过

---

1 commit (2f4d227), 96 files, +5836/-263. All tasks complete.
