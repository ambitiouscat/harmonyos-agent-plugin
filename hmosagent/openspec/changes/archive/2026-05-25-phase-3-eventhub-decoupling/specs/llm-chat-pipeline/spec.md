## llm-chat-pipeline

### Purpose
Full-stack chat data flow from ArkTS user input through Rust HTTP client to LLM API and back as streaming text deltas rendered in the ArkUI ChatView.

### Requirements

- **REQ-LLM-001**: SettingsPage SHALL provide provider selection (134+ from ProviderLoader), model dropdown (scrollable, full model list), API Key input, and Base URL with auto-fill from provider data
- **REQ-LLM-002**: Configuration SHALL be persisted to Preferences and injected into Rust core via `bridge.call("configure", {...})`
- **REQ-LLM-003**: `ChatView.initBridge()` SHALL read saved settings on startup and auto-configure the bridge if API key exists
- **REQ-LLM-004**: After settings saved, `handleSettingsSaved()` SHALL `await initBridge()` before closing the settings panel
- **REQ-LLM-005**: `bridge.call("chat", {messages})` SHALL return immediately; the chat handler SHALL spawn a background thread
- **REQ-LLM-006**: The Rust thread SHALL use `ureq` to POST to `{base_url}/chat/completions` with `stream: true`
- **REQ-LLM-007**: SSE chunks SHALL be parsed by splitting the response body on `\n\n` and extracting `data:` lines
- **REQ-LLM-008**: Each text delta SHALL be wrapped as `{"type":"text","text":"..."}` and sent through the stream callback (`STREAM_CB`)
- **REQ-LLM-009**: Chunks SHALL be emitted with 20ms inter-chunk delay to allow ArkUI render cycles (streaming visual effect)
- **REQ-LLM-010**: Completion SHALL send eventType=1, errors SHALL send eventType=2 through the stream callback
- **REQ-LLM-011**: `applyDelta()` SHALL append text to the last assistant message's `ArkContentPart`, triggering `@ObservedV2` re-render
- **REQ-LLM-012**: `SseStreamController` (ArkTS) SHALL exist as fallback for non-Rust HTTP paths

### Implementation

- **ArkTS**: `ChatView.ets` initBridge/handleSend/applyDelta, `SettingsPage.ets` saveConfig/loadConfig, `SseStreamController.ets` SSE parser with setTimeout stagger
- **C++/NAPI**: `native_bridge.cpp` AgentCall (generic pass-through), InitAgent (registers stream callback + SystemCallbacks)
- **Rust**: `json_router.rs` dispatch("chat"), `agent/chat.rs` chat_completion_ureq(), `sandbox/validate.rs` STREAM_CB + rust_agent_register_stream_cb
