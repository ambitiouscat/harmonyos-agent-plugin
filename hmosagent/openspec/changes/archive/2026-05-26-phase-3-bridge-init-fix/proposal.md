# Proposal: Phase 3 — Bridge 重复初始化修复

## Why

`RustAgentBridge` 在两个不同位置被初始化：

1. `EditorCenterXComponent.afterI3DInit()` 调用 `initWithIo()` → 触发 `nativeBridge.initAgentWithIo()` → Rust `OnceLock.set(callbacks)` 成功
2. `ChatView.initBridge()` 调用 `init()` → 触发 `nativeBridge.initAgent()` → Rust `OnceLock.set()` 再次调用 **失败**（已设置）→ `initialized = false`

结果：`bridgeReady = false` → `handleSend()` 显示 "Please configure API settings" → AI 无回复。

## What

三层幂等化防御：

1. **RustAgentBridge.init()**: 检测 `this.initialized`，若已初始化则仅更新 streamCallback，不重新调用 native API
2. **RustAgentBridge.initWithIo()**: 同上
3. **ChatView.initBridge()**: 检测 `this.bridgeReady`，若已就绪则跳过
4. **ChatView.handleSend()**: 捕获 bridge.call() 异常并展示错误信息

## Scope

- **In**: RustAgentBridge.ets, ChatView.ets
- **Out**: C++ native_bridge.cpp（审计确认无需修改）、Rust core（OnceLock 设计不变）
- **Impact**: 仅 ArkTS 层，不影响 native 或 Rust 代码

## Impact

- **New spec**: bridge-lifecycle
- **Risk**: 无 — 纯 JS 层守卫，不改变任何 native 合约
