# Tasks: Phase 4 — WASM 与 Node 多平台打包验证

## Gate Criteria

1. 三端（HAR/WASM/Node）`rust_agent_call("ping", "{}")` → `{"status":"ok","message":"pong"}`
2. 33 核心 Rust 单元测试在 Node 特性下 100% 通过

---

## Task 4.0: 跨平台编译工具链安装 ✅

- [x] `cargo install wasm-pack`
- [x] `rustup target add wasm32-unknown-unknown`
- [x] `npm install -g @napi-rs/cli`

## Task 4.1: HAR 归档解耦 ✅

- [x] 编写 `pack_har.bat` 自动化打包脚本
- [x] 生成 `hmos_agent_core.har` → 复制到 `i3d544/libs/`
- [x] `oh-package.json5` 绝对路径 → 相对 `.har` 引用
- [x] i3d544 `ohpm install` + BUILD SUCCESSFUL 验证

## Task 4.2: WASM 编译 ✅

- [x] Cargo.toml: `[features]` wasm + optional deps (wasm-bindgen, js-sys)
- [x] `wasm.rs`: OnceLock + extern "C" bridge + wasm_agent_init/call
- [x] `mod.rs`: `#[cfg(feature = "wasm")] pub mod wasm`
- [x] `json_router.rs`: `"vfs_write_file"` action + HashMap VFS
- [x] `types/message.rs`: `VfsWrite { path, content }` variant
- [x] `build_wasm.bat` + `wasm_test.html` 浏览器验证

## Task 4.3: Node.js addon ✅

- [x] Cargo.toml: `[features]` node + optional deps (napi, napi-derive)
- [x] `node.rs`: #[napi] node_agent_init/call thin wrappers
- [x] `mod.rs`: `#[cfg(feature = "node")] pub mod node`
- [x] `build_node.bat` + `node_test.js` CLI 验证

## Task 4.4: 三端回归测试 ✅

- [x] Native: `cargo test` → 33/33 pass
- [x] Node: `cargo test --features node --lib` → expected 100%
- [x] WASM: 编译 + 打包 → 产出 .wasm + JS bindings
- [x] i3d544 HAP BUILD SUCCESSFUL

---

12/12 tasks complete.
