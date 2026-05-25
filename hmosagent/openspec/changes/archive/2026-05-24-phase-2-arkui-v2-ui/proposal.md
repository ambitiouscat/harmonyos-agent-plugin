## Why

Phase 1 delivered a Rust-side AI brain and Phase 0 physical C ABI channel, but the HarmonyOS UI was a bare 6-button test page. Phase 2 transforms this into a production-quality chat interface — structured stream rendering, ArkUI V2 reactive state, native Markdown display without WebView, and persistent user settings. Without this, the Agent has no usable frontend.

## What Changes

- **Stream protocol upgrade**: Rust `pipeline.rs` + `validate.rs` now emit ContentPart JSON deltas (`{"type":"text","text":"H"}`) instead of raw characters, enabling structured multi-part message rendering
- **10 ArkUI V2 components**: MarkdownParser, MarkdownView (native Text/Span, no WebView), ThinkingPanel (glassmorphism collapse card), ToolCard (tool call/result display), MessageBubble (polymorphic part router), InputBar (send/stop toggle), ChatView (List+ForEach container with applyDelta), ArkMessage+ArkContentPart (@ObservedV2/@Trace models), SettingsPage (Preferences persistence)
- **configure route**: `json_router.rs` gains a `"configure"` action that stores API Key / Base URL / Temperature / Max Tokens into a global `AgentConfig`
- **Entry Index rebuilt**: From 6-button debug console to ChatView-driven stream rendering with @State-driven delta updates

## Capabilities

### New Capabilities
- `structured-stream-protocol`: ContentPart JSON delta format for cross-boundary structured streaming
- `arkui-v2-components`: Reusable ArkUI V2 component library (MarkdownView, ThinkingPanel, ToolCard, MessageBubble, InputBar, ChatView)
- `settings-persistence`: Preferences-based user configuration with cold-start injection into Rust core

### Modified Capabilities
- `rust-agent-core`: New configure route, upgraded stream output from raw chars to ContentPart JSON
- `harmonyos-project-scaffold`: New UI component tree, SettingsPage, refactored Entry Index

## Impact

- **Rust**: `json_router.rs` (+20 lines configure handler), `validate.rs` (stream output format change), `pipeline.rs` (already designed for structured output)
- **ArkTS**: 10 new files in `hmos_agent_core/src/main/ets/ui/`, 1 new SettingsPage, HAR Index.ets updated with 5 exports, Entry Index.ets rewritten
- **Build**: Both arches verified, HAP deployed and structured delta rendering confirmed on emulator
