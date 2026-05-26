## Architecture

### 初始化时序

```
EditorCenterXComponent.afterI3DInit()
  → RustAgentBridge.getInstance().initWithIo(noopCb, dummyIo)
  → nativeBridge.initAgentWithIo(streamFn, ioFn)
  → Rust OnceLock.set(callbacks) ✅
  → this.initialized = true

ChatView.aboutToAppear()
  → initBridge()
  → if (this.bridgeReady) return  ← 守卫
  → bridge.init(applyDeltaCb)
  → if (this.initialized) {        ← 守卫
        this.streamCallback = applyDeltaCb
        return true
    }
  → bridgeReady = true
  → bridge.call('configure', {...})
```

### 幂等化策略

| 方法 | 守卫条件 | 行为 |
|------|---------|------|
| `init()` | `this.initialized` | 仅更新 streamCallback，返回 true |
| `initWithIo()` | `this.initialized` | 仅更新 streamCallback + ioCallback，返回 true |
| `initBridge()` | `this.bridgeReady` | 跳过整个初始化流程 |

### 错误处理

`handleSend()` 新增 try/catch 包裹 `bridge.call('chat', ...)`:
- 若返回 `{"status":"error"}` → 在聊天中展示错误消息
- 若抛出异常 → 展示 `Chat failed: {e}` 并重置 `isStreaming`

## Key Design Decisions

1. **不修改 native 层**: C++ `native_bridge.cpp` 的 `initAgent` 和 `initAgentWithIo` 保持原样。幂等化在 ArkTS 层通过 JavaScript 守卫实现。
2. **不修改 Rust OnceLock**: `OnceLock.set()` 的"仅一次"语义是正确的。问题在于 ArkTS 层不应二次调用。
3. **回调热替换**: `this.streamCallback` 是可变的。native 闭包通过 `this` 捕获，因此延迟绑定始终解析到当前回调。
