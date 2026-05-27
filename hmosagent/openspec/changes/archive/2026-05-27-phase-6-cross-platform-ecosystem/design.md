# Design: Phase 6 — 跨平台生态与未完残片彻底贯通

## Architecture: Rust Core 强内聚

Phase 6 将原计划在 ArkTS 宿主端实现的重度逻辑，按跨平台内聚原则全部收拢至 Rust Core：

```
┌──────────────────────────────────────────────────┐
│          ArkTS 宿主层 (极轻 UI 渲染)              │
│   - ConversationList | CommandPalette | Themes    │
│   - ChatView | InputBar | MarkdownView | Settings │
└──────────────────────┬───────────────────────────┘
                       │ FFI / C ABI
                       ▼
┌──────────────────────────────────────────────────┐
│           Rust 跨平台核心层 (agent_core)          │
│                                                  │
│   - SessionManager (JSON 持久化)                  │
│   - SkillsRegistry (斜杠命令注册/派发)            │
│   - Tools (File/Web/MCP/SubAgent)                │
│   - SSE Reconnect (指数退避 retry)               │
│   - ReAct Loop Engine + Pipeline + RAG           │
└──────────────────────────────────────────────────┘
```

## Decision Records

### D1: Session CRUD 直接解析 JSON
**Why**: `AgentRequest` 使用 `#[serde(untagged)]` 导致 `CreateSession{title}` / `DeleteSession{session_id}` / `LoadSession{session_id}` 单字段变体无法区分。
**Decision**: session 路由直接从 `args_json` 用 `serde_json::Value["key"]` 解析，不走 `AgentRequest` deser。
**Impact**: `json_router.rs` session 路由 (init_session, load_session, create_session, list_sessions, delete_session, save_session)

### D2: SkillsRegistry 使用 LazyLock
**Why**: `static RwLock::new(SkillsRegistry::new())` 需要 `const fn new()`，但 `SkillsRegistry::new()` 调用 `register_builtins()` 非 const。
**Decision**: `LazyLock<RwLock<SkillsRegistry>>` 延迟初始化。
**Impact**: `agent/skills.rs`

### D3: WebTools 委托宿主 HTTP
**Why**: 避免捆绑 HTTP 客户端到 Rust 核心（ureq+rustls 在 wasm32 不兼容），通过 FFI `post_fn` 回调代理到宿主原生 HTTP 栈。
**Decision**: WebTools 的 `post_fn` 字段接收 `extern "C" fn(*const c_char, *const c_char) -> *mut c_char`，由 ArkTS `@ohos.net.http` 或 Node.js `fetch` 实现。
**Impact**: `tools/web.rs`, 无需 wasm32 HTTP 依赖

### D4: McpBridge 同步 std::process + stdio
**Why**: 严禁 `tokio` 异步事件循环。MCP 协议本身是 JSON-RPC over stdio，同进程 spawn + pipe 即可。
**Decision**: 使用 `std::process::Command` spawn MCP server，`stdin/stdout` pipe 读写 JSON-RPC。
**Impact**: `tools/mcp.rs`, `#[cfg(not(target_arch = "wasm32"))]`

### D5: SSE 去重在 Rust 侧
**Why**: HTTP SSE 重连后 LLM 重新生成，首部内容与中断前重叠。若去重在 ArkTS 侧，需要维护全局去重状态。
**Decision**: `chat.rs` 的 `find_overlap()` 用最后 200 字符做前缀匹配，跳过已发送内容。ArkTS 侧只需识别 `type: "reconnect"` 事件更新状态栏。
**Impact**: `agent/chat.rs`, `ChatView.ets` applyDelta

### D6: 设置页浮层化
**Why**: 原来 SettingsPage 替换 Scroll 区域导致销毁重建，滚动位丢失。
**Decision**: SettingsPage 作为 Stack overlay 全屏覆盖，聊天 Scroll 始终挂载。顶部加标题栏 + ✕ 关闭。
**Impact**: `ChatView.ets` build()

### D7: Session 延迟入库
**Why**: 用户每次打开应用自动创建 session，若未产生对话则留下空文件。
**Decision**: 启动/新建时仅生成本地 ID (`sessionPending=true`)，`saveCurrentSession()` 检测到 `sessionPending` 时先 `create_session` 再 `save_session`。`messages.length === 0` 不保存。
**Impact**: `ChatView.ets` createSession, saveCurrentSession, genLocalId

### D8: 主题切换实时响应
**Why**: `@StorageLink`/`@StorageProp` 在 `@ComponentV2` 中不可用。MarkdownView 缓存旧 ThemeConfig 导致切换后不更新。
**Decision**: MarkdownView 每次 `build()` 通过 `tc()` 实时读取 `getThemeLoader().getCurrent()`。ChatView ForEach key 加入 `themeVersion` 强制重渲染。
**Impact**: `MarkdownView.ets`, `ChatView.ets`

## Target State

```
Phase 0: 交叉编译 + C ABI           ✅
Phase 1: Rust 大脑业务 + IO 注入    ✅
Phase 2: ArkUI V2 状态绑定          ✅
Phase 3: EventHub + Godot 解耦      ✅
Phase 4: WASM + Node 多平台         ✅
Phase 5: 极限性能调优               ✅
Phase 6: 跨平台生态与残片贯通        ✅ (current)
          Session / Skills / Tools
          Drawer / Palette / Theme
          SSE Reconnect / Offline
          Icons / Scripts / SDK
```
