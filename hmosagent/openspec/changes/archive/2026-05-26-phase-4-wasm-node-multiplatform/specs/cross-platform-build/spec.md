## cross-platform-build

### Purpose
Enable HmosAgent Rust core to compile and run across three platforms — HarmonyOS (HAR), Web (WASM), and Node.js — through unified C ABI thin wrappers and conditional compilation.

### Requirements

- **REQ-CPB-001**: `Cargo.toml` SHALL define `wasm` and `node` features with appropriate optional dependencies
- **REQ-CPB-002**: WASM platform SHALL bridge JS `Function` callbacks to C ABI via `OnceLock` + `extern "C"` bridge functions
- **REQ-CPB-003**: Node platform SHALL use napi-rs `#[napi]` wrappers with zero-callback design (native I/O)
- **REQ-CPB-004**: `#[cfg(not(target_arch = "wasm32"))]` SHALL exclude chat, rag, search modules from WASM builds
- **REQ-CPB-005**: WASM platform SHALL provide a virtual filesystem (`vfs_write_file` action) for RAG/search operations
- **REQ-CPB-006**: All three platforms SHALL pass the ping gate: `{"status":"ok","message":"pong"}`
- **REQ-CPB-007**: HAR dependency SHALL use relative `file:./libs/hmos_agent_core.har` path (not absolute)
- **REQ-CPB-008**: `cargo test` SHALL pass 33/33 core tests on native target
