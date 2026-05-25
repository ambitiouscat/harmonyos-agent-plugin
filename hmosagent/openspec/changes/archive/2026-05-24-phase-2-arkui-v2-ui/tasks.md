## 1. Rust Stream Protocol Upgrade

- [x] 1.1 Add `configure` action to `json_router.rs` with global `AGENT_CONFIG` storage
- [x] 1.2 Upgrade `validate.rs` `start_stream_sim` to emit ContentPart JSON deltas
- [x] 1.3 Verify `cargo test` passes (32 tests)

## 2. ArkUI V2 Reactive Data Models

- [x] 2.1 Create `ArkContentPart.ets` with `@ObservedV2` and `@Trace` properties
- [x] 2.2 Create `ArkMessage.ets` with `@ObservedV2`, `@Trace parts` array
- [x] 2.3 Confirm single-layer throttling (no TS-side debounce, Rust 16ms only)

## 3. MarkdownView — Native Rendering

- [x] 3.1 Create `MarkdownParser.ts` (4 block types: heading, paragraph, code_block, bullet_list)
- [x] 3.2 Create `MarkdownView.ets` using pure ArkUI Text/Span (no WebView)
- [x] 3.3 Add code block copy-to-clipboard function

## 4. ThinkingPanel + ToolCard

- [x] 4.1 Create `ThinkingPanel.ets` with glassmorphism styling and collapse animation
- [x] 4.2 Create `ToolCard.ets` with status dot, tool name, and collapsible output
- [x] 4.3 Bind `@Trace collapsed` for both components

## 5. ChatView + InputBar + MessageBubble + SettingsPage

- [x] 5.1 Create `InputBar.ets` with send/stop toggle
- [x] 5.2 Create `MessageBubble.ets` as polymorphic part router
- [x] 5.3 Create `ChatView.ets` with `applyDelta()` and `List` + `ForEach` rendering
- [x] 5.4 Create `SettingsPage.ets` with Preferences persistence (API Key, Base URL, Temperature, Max Tokens)
- [x] 5.5 Update HAR `Index.ets` with 5 component exports
- [x] 5.6 Rewrite Entry `Index.ets` with `@State messages` and structured delta handler
- [x] 5.7 Fix Entry Index V1/V2 rendering — embed inline chat UI

## 6. Build & Deploy Verification

- [x] 6.1 Cross-compile both arches (x86_64 + aarch64)
- [x] 6.2 `hvigorw assembleHap`: BUILD SUCCESSFUL (0 errors)
- [x] 6.3 Deploy to emulator, verify structured delta rendering on UI
- [x] 6.4 Git commit with descriptive message

## 7. OpenSpec Documentation

- [x] 7.1 Phase 2 implementation plan review (3 issues fixed)
- [x] 7.2 OpenSpec artifacts: proposal, design, 5 specs (3 new + 2 modified), tasks
