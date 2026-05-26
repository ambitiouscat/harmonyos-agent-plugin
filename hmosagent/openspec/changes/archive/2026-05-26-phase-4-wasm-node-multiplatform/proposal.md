# Proposal: Phase 4 — WASM 与 Node 多平台打包验证

## Why

HmosAgent 必须在三端可用：HarmonyOS (HAR)、浏览器 (WASM)、桌面 CLI (Node.js)。Rust 核心通过统一 C ABI (`rust_agent_call`) 已经提供了平台无关接口，但需要为每个平台构建薄适配层，同时清偿 Phase 3 遗留的 HAR 绝对路径技术债。

## What

1. **HAR 归档解耦**: 将 `oh-package.json5` 的绝对路径依赖替换为本地 `.har` 文件引用
2. **WASM 编译**: 通过 `wasm-pack` + features 隔离，将 Rust 核心编译为 `.wasm`
3. **Node.js addon**: 通过 `napi-rs` 构建原生 Node 模块
4. **条件编译**: `#[cfg(not(target_arch = "wasm32"))]` 排除平台不兼容模块 (chat/rag/search)
5. **门禁验证**: 三端 ping 全通 + 33 核心测试回归 100%

## Scope

- **In**: Cargo features, wasm.rs/node.rs 薄封装, build scripts, test harnesses, pack_har.bat
- **Out**: WSL 交叉编译优化 (已有), ArkTS 层变更 (无)
- **Platforms**: HarmonyOS (HAR), Web (WASM), Node.js desktop

## Impact

- **New spec**: cross-platform-build
- **New code**: ~10 files across rust_core/ + hmosagent/
- **Risk**: 低 — 所有平台共享同一 C ABI，适配层 < 80 行/平台
