# Tasks: Phase 3 — Bridge 重复初始化修复

## Task 1: RustAgentBridge.init() 幂等化 ✅

- [x] 在 `init()` 顶部添加 `if (this.initialized) { this.streamCallback = onChunk; return true; }`
- [x] 验证：二次调用返回 true，不触发 nativeBridge.initAgent()

## Task 2: RustAgentBridge.initWithIo() 幂等化 ✅

- [x] 在 `initWithIo()` 顶部添加 `if (this.initialized) { ...; return true; }`
- [x] 验证：二次调用返回 true，不触发 nativeBridge.initAgentWithIo()

## Task 3: ChatView.initBridge() 守卫 ✅

- [x] 在 `initBridge()` 顶部添加 `if (this.bridgeReady) { return; }`
- [x] 验证：二次调用为 no-op，不读取 preferences 或调用 bridge.init()

## Task 4: ChatView.handleSend() 错误展示 ✅

- [x] 用 try/catch 包裹 `bridge.call('chat', ...)`
- [x] 解析响应 JSON，检查 `status === 'error'`
- [x] 在聊天中添加错误消息气泡
- [x] 出错时重置 `isStreaming = false`

## Task 5: HAR 构建 ✅

- [x] `build_project(module=hmos_agent_core@default)` → BUILD SUCCESSFUL

## Task 6: HAP 构建 ✅

- [x] 复制 HAR → i3d544/libs/
- [x] `build_project(module=hap_editor@default)` → BUILD SUCCESSFUL (46s)

## Task 7: 真机验证 ✅

- [x] hilog 确认 `initAgent` 仅调用一次
- [x] `agentCall action=chat` 正常工作
- [x] 零错误，对话可用

---

12/12 tasks complete.
