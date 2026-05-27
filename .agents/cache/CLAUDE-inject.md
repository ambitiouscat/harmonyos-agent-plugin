---
generated_at: 2026-05-27
---

# CLAUDE.md Inject Fragment

## Project: hmosagent — AI Agent HarmonyOS Plugin

Cross-platform AI agent with **HarmonyOS NEXT** UI (ArkTS), **C++ NAPI bridge**, and **Rust core engine**. Also compiles to WASM and Node.js native addon.

## Key Directories
- `hmosagent/hmos_agent_core/src/main/ets/` — ArkTS UI source (28 .ets files)
- `hmosagent/hmos_agent_core/src/main/cpp/` — C++ NAPI bridge (native_bridge.cpp + CMakeLists.txt)
- `rust_core/agent_core/src/` — Rust core engine (~15 .rs modules)
- `rust_core/agent_core/src/agent/` — Agent runtime (chat, loop_engine, session, skills, rag, search)
- `rust_core/agent_core/src/tools/` — Tools (file, mcp, subagent, web)
- `scripts/` — Build orchestration (cross-compile.sh, build-all.sh)

## Tech Stack
- **ArkTS/ArkUI** (HarmonyOS SDK 6.0.2, API 22) — UI layer with @ObservedV2 state management
- **C++17** — NAPI bridge using napi_threadsafe_function
- **Rust 2021** — Core engine (serde, ureq HTTP, sha2, ripgrep search, BM25 RAG)
- **Build**: Hvigor (HarmonyOS), Cargo (Rust), CMake+Ninja (native bridge), WSL2 (cross-compile)

## Architecture Pattern
```
ArkTS UI → NAPI → C++ Bridge → C FFI → Rust json_router → agent/tools modules
```
All cross-boundary communication is JSON via `rust_agent_call(action, json_args)`.

## Naming
- ArkTS: PascalCase components, camelCase methods
- Rust: snake_case modules, PascalCase types
- Files: snake_case for Rust/C++, PascalCase for ArkTS

## Testing
- Rust: `#[cfg(test)]` inline unit tests (~35) + `tests/ffi_integration.rs` (7 integration tests)
- Run: `cargo test` from `rust_core/`
- HarmonyOS: DevEco Studio build via hvigor

