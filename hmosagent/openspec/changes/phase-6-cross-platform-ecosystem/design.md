## Design: Phase 6 — 跨平台生态与未完残片彻底贯通

### Architecture

```
┌──────────────────────────────────────────┐
│        ArkTS 宿主层 (极轻 UI)             │
│   - ConversationList | CommandPalette     │
│   - ChatView | InputBar | MarkdownView   │
│   - ThemeLoader | SettingsPage (overlay) │
└────────────────────┬─────────────────────┘
                     │ FFI / C ABI
                     ▼
┌──────────────────────────────────────────┐
│         Rust 跨平台核心 (agent_core)      │
│                                          │
│   - SessionManager (JSON 持久化)          │
│   - SkillsRegistry  (斜杠命令注册/派发)   │
│   - Tools (File/Web/MCP/SubAgent)        │
│   - SSE Reconnect (指数退避 retry)       │
│   - ReAct Loop + Pipeline + RAG         │
└──────────────────────────────────────────┘
```

### Key Decisions

1. **Session CRUD 直接解析 JSON** — AgentRequest untagged enum 无法区分单字段变体，改为 `serde_json::Value["key"]` 直接读取
2. **SkillsRegistry LazyLock** — `RwLock::new(SkillsRegistry::new())` 需要 const fn，改用 `LazyLock` 延迟初始化
3. **WebTools 委托宿主 HTTP** — 避免 ureq+rustls wasm32 不兼容，通过 FFI post_fn 回调代理到宿主原生栈
4. **McpBridge 同步 stdio** — 严禁 tokio，使用 `std::process::Command` + stdin/stdout pipe
5. **SSE 去重在 Rust 侧** — `find_overlap()` 最后 200 字符前缀匹配，ArkTS 侧只识别 `type: "reconnect"` 更新状态栏
6. **SettingsPage 浮层化** — Stack overlay 全屏覆盖，不销毁 Scroll，避免滚动位丢失
7. **Session 延迟入库** — 启动/新建仅本地 ID，首条消息才 `create_session`，空 session 不保存
8. **主题实时响应** — V2 无 @StorageLink，MarkdownView 每次 `build()` 实时读取 ThemeLoader

### Platform Compatibility

| Module | HAR | WASM | Node | Notes |
|--------|-----|------|------|-------|
| session | | | | std::fs, runtime fallback on wasm |
| skills | | | | pure Rust, no platform deps |
| file | | | | `#[cfg(not(wasm32))]` std::fs |
| mcp | | | | `#[cfg(not(wasm32))]` std::process |
| subagent | | | | `#[cfg(not(wasm32))]` std::thread |
| web | | | | c_char compatible for wasm32 |
| chat (ureq) | | | | `#[cfg(not(wasm32))]` conditional dep |
