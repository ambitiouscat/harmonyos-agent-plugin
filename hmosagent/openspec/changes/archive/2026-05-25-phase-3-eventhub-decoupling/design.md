## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    i3d544 Editor (HAP)                          │
│                                                                 │
│  ┌──────────────────────┐    ┌──────────────────────────────┐  │
│  │  Web (dist/index)    │    │  ChatView (HAR)              │  │
│  │  - scene tree        │    │  ┌────────────────────────┐  │  │
│  │  - inspector         │    │  │ SettingsPage            │  │  │
│  │  - viewport          │    │  │ - 134 providers         │  │  │
│  └──────┬───────────────┘    │  │ - model dropdown        │  │  │
│         │ MessagePort        │  │ - search filter         │  │  │
│         ▼                    │  └────────┬───────────────┘  │  │
│  ┌──────────────────────┐    │           ▼                   │  │
│  │  Index.ets           │    │  ┌────────────────────────┐  │  │
│  │  - EventHub          │    │  │ ChatView               │  │  │
│  │  - WEB2CPP/CPP2WEB   │    │  │ - handleSend()         │  │  │
│  │  - isVibeMode toggle  │    │  │ - applyDelta()         │  │  │
│  └──────┬───────────────┘    │  │ - RustAgentBridge       │  │  │
│         │ cpp2web callback    │  └────────┬───────────────┘  │  │
│         ▼                    │           │ NAPI              │  │
│  ┌──────────────────────┐    │           ▼                   │  │
│  │  EditorCenterXComp   │    │  ┌────────────────────────┐  │  │
│  │  - LogFilter(20)     │    │  │ Rust Core (agent_core) │  │  │
│  │  - cpp2web dispatch   │    │  │ - json_router          │  │  │
│  └──────┬───────────────┘    │  │ - agent::chat          │  │  │
│         │ TSFC               │  │ - agent::abort         │  │  │
│         ▼                    │  │ - ureq → LLM API       │  │  │
│  ┌──────────────────────┐    │  └────────────────────────┘  │  │
│  │  Godot C++           │    └──────────────────────────────┘  │
│  │  - editor_node       │                                       │
│  │  - arktsTasks queue   │                                       │
│  │  - SubDataThread      │                                       │
│  └──────────────────────┘                                       │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flows

### Flow A: Godot → Web (scene updates)
```
Godot send_data_to_browser()
  → arktsTasks.push({event:"data_transform", data:json})
  → SubDataThread() wakes → ThreadSafeDataCallJs()
  → EditorCenterXComponent.cpp2web()
  → LogFilter.filter(data)
  → EventHub.emit(CPP2WEB, filtered)
  → Index.ets ports[1].postMessageEvent()
  → Web receives via MessagePort
```

### Flow B: Web → Godot (user actions)
```
Web MessagePort.postMessage()
  → Index.ets ports[1].onMessageEvent
  → EventHub.emit(WEB2CPP, rawJson)
  → EditorCenterXComponent.on(WEB2CPP)
  → plugin.frontSlot(data)
  → C++ processWebToCppMsgThread()
  → EditorNode::front_slot() → dispatch to Scene/Inspector/Node docks
```

### Flow C: Chat → LLM → Response (AI pipeline)
```
ChatView.handleSend(text)
  → bridge.resetAbort()
  → bridge.call("chat", {messages})
  → NAPI: AgentCall → rust_agent_call
  → json_router::dispatch("chat")
  → spawn thread:
      → chat_completion_ureq(config, messages, on_chunk)
      → ureq::post(base_url/chat/completions, stream:true)
      → read_to_string → parse SSE events
      → for each delta:
          → on_chunk({"type":"text","text":content})
          → thread::sleep(20ms)
      → on_chunk("", 1) on completion
  → STREAM_CB → C++ OnChunkBridge
  → napi_call_threadsafe_function(g_stream_tsfn)
  → DeliverChunkToJS → js_callback(data, eventType)
  → ChatView.applyDelta(data) → ArkMessage.parts[].text += delta
  → @ObservedV2/@Trace triggers UI re-render
```

### Flow D: Settings → Configure
```
SettingsPage.saveConfig()
  → preferences.put(key, value) × 5
  → onSettingsSaved()
  → ChatView.handleSettingsSaved()
  → await initBridge()
  → bridge.init(streamCallback)
  → bridge.call("configure", {api_key, base_url, model, max_tokens:32000})
  → Rust: AGENT_CONFIG.write() = cfg
```

## Key Design Decisions

1. **No direct C++↔ArkTS coupling**: All cross-layer communication goes through EventHub with string keys (WEB2CPP/CPP2WEB). Adding new event types requires no interface changes.

2. **ChatView is self-contained**: Imported as a single component from HAR. No props needed — it internally manages RustAgentBridge (singleton), Preferences, and the stream pipeline.

3. **ArkTS HTTP fallback preserved**: The `SseStreamController` class exists for fallback/alternative HTTP paths. Primary path is Rust ureq.

4. **Cross-compilation dual-compiler strategy**: x86_64 uses gcc for ring C code (native Linux, no cross-compile issues). aarch64 uses OHOS clang.exe wrappers. Both link via OHOS clang.

5. **LogFilter at host boundary**: Noise filtering happens in the host project (hap_editor), not in the HAR, keeping the HAR portable.

6. **ABORT_FLAG independent module**: `agent::abort` is a leaf module with zero dependencies, avoiding reverse-dependency issues.
