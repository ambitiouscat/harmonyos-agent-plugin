## Why

Phase 2 delivered a standalone chat UI. Phase 3 embeds that chat UI into the Godot-based i3D editor as a "VibeCoding" mode, creating a zero-coupling EventHub bridge between Godot C++ and the HmosAgent HAR, adding abort control to the Rust core, filtering noisy Godot console output, and establishing a working LLM chat pipeline from UI input through to AI response.

## What Changes

- **EventHub bidirectional message bus**: Godot C++ `node_selected` events + scene updates flow through `cpp2web` → EventHub → Web MessagePort, while Web UI actions flow back via `web2cpp` → Godot `frontSlot()`. All communication is decoupled through string-keyed events with no direct C++↔ArkTS coupling.
- **VibeCoding mode**: `Index.ets` gains a top toggle bar switching between Normal (Web 100%) and VibeCoding (Web 65% + ChatView 35%) Row layout. ChatView is imported from the hmos_agent_core HAR.
- **HAR barrel refactor**: 6 `index.ets` barrel files created for clean cross-directory imports, enabling HAR binary consumption from the host i3d544 project.
- **Rust abort chain**: `agent::abort` module with global `AtomicBool ABORT_FLAG`, wired into `json_router` (`abort`/`reset_abort` actions) and `loop_engine` (`can_continue()` check).
- **LogFilter**: Sliding-window dedup filter stripping Vulkan/texture/frame-queue noise from Godot's `data_transform` pipeline before forwarding to Web.
- **LLM chat pipeline**: `ChatView.handleSend()` → `bridge.call("chat")` → NAPI → Rust `json_router` → `thread::spawn` → `ureq::post()` → SSE parse → stream callback → `napi_threadsafe_function` → `applyDelta()`. SettingsPage with 134-provider dropdown, model selector, and Preferences persistence.
- **WSL cross-compilation**: `gcc` (x86_64 ring C code) + OHOS `clang.exe` (aarch64 + linking) via WSL2, with build scripts for one-command full builds.

## Capabilities

### New Capabilities
- `event-hub-bridge`: Zero-coupling bidirectional message pipeline (Godot C++ ↔ ArkTS EventHub ↔ Web MessagePort)
- `log-filter`: Regex-based noise stripping + dedup sliding window for Godot console
- `abort-chain`: AtomicBool-based abort/reset_abort control for Rust streaming operations
- `vibecoding-ui`: Toggleable split-pane layout embedding ChatView alongside 3D editor
- `llm-chat-pipeline`: Full-stack chat flow (ArkTS → NAPI → Rust/ureq → SSE → stream callback → ArkTS applyDelta)
- `cross-compile-wsl`: WSL2-based cross-compilation for musl targets with ureq/ring/rustls support

### Modified Capabilities
- `rust-agent-core`: New `chat` action, `chat_completion_ureq()` function, `agent::chat` module
- `harmonyos-project-scaffold`: HAR barrel files, dependency injection via oh-package.json5

## Impact

- **Rust**: `agent/chat.rs` (70 lines, new), `json_router.rs` (+60 lines chat handler), `agent/abort.rs` (new), `Cargo.toml` (+ureq dep), `.cargo/config.toml` (WSL cross-compile setup)
- **C++**: `editor_node.cpp` (+25 lines node_selected emission), `native_bridge.cpp` (unchanged, validated as generic pass-through)
- **ArkTS**: `ChatView.ets` (rewritten handleSend + initBridge), `SettingsPage.ets` (provider/model dropdowns, search filter), `SseStreamController.ets` (new, fallback streaming), `ProviderLoader.ets` (UTF-8 decode fix), 6 barrel files, `Index.ets` (VibeCoding layout), `EditorCenterXComponent.ets` (LogFilter integration)
- **Scripts**: 5 new build scripts in `scripts/` (env-setup, cross-compile, build-hap, build-all, sync-har)
- **Build**: Dual-arch .a (54MB/53MB) with ureq+ring+rustls, HAR assembly, i3d544 integration
