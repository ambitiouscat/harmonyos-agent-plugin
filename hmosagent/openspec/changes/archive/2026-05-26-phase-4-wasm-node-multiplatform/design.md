## Architecture

### Multi-Platform Build Topology

```
                    ┌──────────────────────────────────────┐
                    │   Rust Core (agent_core)              │
                    │   - rust_agent_init(callbacks)       │
                    │   - rust_agent_call(action, args)    │
                    │   - Cargo features: wasm, node        │
                    └──────────────┬───────────────────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        ▼                          ▼                          ▼
┌──────────────────┐   ┌──────────────────┐   ┌──────────────────┐
│  Static lib (.a) │   │  WASM (wasm-pack) │   │  Node (napi-rs)  │
│  aarch64-musl    │   │  wasm.rs          │   │  node.rs         │
│  x86_64-musl     │   │  OnceLock bridge  │   │  #[napi] wrappers│
└────────┬─────────┘   └────────┬─────────┘   └────────┬─────────┘
         │                      │                      │
         ▼                      ▼                      ▼
┌──────────────────┐   ┌──────────────────┐   ┌──────────────────┐
│  HarmonyOS App   │   │  Browser / Web   │   │  Node.js CLI     │
│  .har → HAP      │   │  wasm_agent_call │   │  node_agent_call │
└──────────────────┘   └──────────────────┘   └──────────────────┘
```

### WASM OnceLock Bridge

WASM 环境的核心挑战：JS `Function` 无法直接转换为 `extern "C" fn` 裸函数指针。

解决方案：
1. `OnceLock<js_sys::Function>` 持有 JS 回调引用 (防止 GC)
2. `extern "C" fn wasm_on_chunk_bridge` 作为 C ABI 桥接函数
3. Bridge 函数从 OnceLock 取出 JS Function → `call2()` 调用 → 数据回到 JS

### Node Zero-Callback Design

Node 环境拥有完整 `std::fs` + `ureq` 能力。`node_agent_init` 仅需传入 config JSON，无需 JS 侧 IO 回调。`SystemCallbacks` 的 IO 字段为 `None`。

### Conditional Compilation

| 模块 | wasm32 | native | 原因 |
|------|--------|--------|------|
| `agent::chat` | ✗ | ✓ | ureq 不支持 wasm32 |
| `agent::rag` | ✗ | ✓ | `std::fs` 不可用 |
| `agent::search` | ✗ | ✓ | `ignore`/`grep` 不支持 wasm32 |
| `agent::wasm` | ✓(feature) | ✗ | wasm-bindgen 仅 wasm 目标 |
| `agent::node` | ✗ | ✓(feature) | napi 仅 native 目标 |
| `ffi::rust_agent_search` | ✗ | ✓ | 文件系统依赖 |
| `ffi::rust_agent_scan_dir` | ✗ | ✓ | 文件系统依赖 |
| `json_router "chat"` | ✗ | ✓ | 通过 spawn thread + ureq |
| `json_router "vfs_write_file"` | ✓ | ✗ | WASM 虚拟文件系统 |

## Key Design Decisions

1. **统一 C ABI 薄包装**: 所有平台共享 `rust_agent_call` 入口，适配层仅做类型转换
2. **OnceLock 回调桥接**: 解决 JS Closure → C 函数指针的生命周期问题
3. **Node 零回调**: 利用原生 I/O 能力，简化桌面端管道
4. **cdylib 全局保留**: wasm-pack 编译需要，musl 构建不受影响
5. **VFS 虚拟文件系统**: WASM 下 RAG/搜索操作代理到内存 HashMap
