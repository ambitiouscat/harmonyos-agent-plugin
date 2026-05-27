---
generated_at: 2026-05-27
last_scan: 2026-05-27
---

# Directory Structure

## Top-Level Layout

```
ai-agent-harmoney/
├── hmosagent/              # HarmonyOS Application (HAP + HAR modules)
├── rust_core/              # Rust Workspace (agent_core crate)
├── scripts/                # Build Orchestration (Bash)
├── reports/                # Generated Reports
├── .agents/cache/          # AI Agent Project Cognition Cache
├── .workspace-session-skill/ # Session Persistence
├── README.md
└── LICENSE (MIT)
```

## HarmonyOS App (`hmosagent/`)

```
hmosagent/
├── build-profile.json5          # App build & signing config
├── oh-package.json5             # Top-level package
├── hvigorfile.ts                # Hvigor entry
│
├── AppScope/
│   ├── app.json5                # Bundle: com.example.hmosagent v1.0.0
│   └── resources/
│
├── entry/                       # [Entry HAP] — Shell application
│   ├── src/main/ets/
│   │   ├── entryability/
│   │   │   └── EntryAbility.ets # App lifecycle → loads Index
│   │   └── pages/
│   │       └── Index.ets        # @Entry page → ChatView
│   └── src/main/module.json5
│
├── hmos_agent_core/             # [Core HAR] — All business logic (zero external deps)
│   ├── Index.ets                # Public API: RustAgentBridge, ChatView, SettingsPage, models
│   ├── BuildProfile.ets
│   ├── src/main/ets/
│   │   ├── RustAgentBridge.ets  # Singleton NAPI bridge (init, call, search, abort, retry)
│   │   ├── pages/
│   │   │   └── SettingsPage.ets # Provider/API key/model/phrase config
│   │   ├── provider/
│   │   │   └── ProviderLoader.ets # 3-tier provider loading (embedded → cache → remote)
│   │   ├── stream/
│   │   │   └── SseStreamController.ets # SSE parser + interleaved timers
│   │   ├── tools/
│   │   │   └── SystemIoImpl.ets  # HTTP POST + file IO for Rust callbacks
│   │   └── ui/
│   │       ├── chat/             # ChatView, MessageBubble, MarkdownView, MarkdownParser
│   │       ├── common/           # ThinkingPanel (collapsible spinner), PhraseLoader
│   │       ├── conversation/     # ConversationList (session drawer, 80% width)
│   │       ├── input/            # InputBar (text + /command), CommandPalette (dropdown)
│   │       ├── models/           # ArkMessage, ArkContentPart (@ObservedV2 + @Trace)
│   │       └── tools/            # ToolCard (status indicator + collapsible output)
│   ├── src/main/cpp/
│   │   ├── CMakeLists.txt        # Links libagent_core.a + ace_napi + hilog
│   │   ├── native_bridge.cpp     # Thread-safe NAPI ↔ Rust bridge
│   │   ├── musl_shim.c           # OHOS musl missing-symbol stubs
│   │   └── agent_core.h          # cbindgen-generated C header
│   └── src/main/resources/rawfile/
│       ├── providers.json        # Embedded LLM provider list
│       └── agent_thinking_phrases.json # Thinking phrase categories
│
├── libs/
│   ├── arm64-v8a/libagent_core.a # Pre-built Rust static lib (physical device)
│   └── x86_64/libagent_core.a    # Pre-built Rust static lib (emulator)
│
├── openspec/changes/             # OpenSpec change records
└── screenshot/                   # Runtime UI dumps & screenshots
```

## Rust Workspace (`rust_core/`)

```
rust_core/
├── Cargo.toml                    # Workspace: members=["agent_core"]
├── cbindgen.toml                 # C header generation config
├── .cargo/config.toml            # OHOS cross-compilation targets
│
└── agent_core/
    ├── Cargo.toml                # staticlib + cdylib + rlib
    ├── src/
    │   ├── lib.rs                # Crate root
    │   ├── ffi.rs                # C ABI: rust_agent_init, rust_agent_call, etc.
    │   ├── json_router.rs        # ~20 action dispatcher
    │   ├── agent/                # Runtime: chat, loop_engine, session, skills, rag, search, context, pipeline, abort, wasm, node
    │   ├── types/message.rs      # ChatMessage, AgentRequest, Message, ContentPart
    │   ├── tools/                # file, mcp, subagent, web
    │   └── sandbox/validate.rs   # Network/file/stream validation
    └── tests/
        └── ffi_integration.rs    # 7 integration tests
```

## Build Scripts (`scripts/`)

```
scripts/
├── build-all.sh                  # Full pipeline: Rust cross-compile → HAP
├── cross-compile.sh              # Rust → OHOS musl (WSL)
├── build-hap.sh                  # HAP via hvigor
├── env-setup.sh                  # DevEco Studio env vars
└── sync-har.sh                   # HAR deploy to i3d544 project
```

## File Counts by Layer

| Layer | Files | Language |
|-------|-------|----------|
| ArkTS UI | 28 | ArkTS/ArkUI |
| C++ Bridge | 3 (.cpp/.c/.h) + 1 CMake | C++17/C |
| Rust Core | ~15 source + 1 test + 3 build | Rust 2021 |
| Build Scripts | 5 .sh + 2 .bat | Bash/Batch |
| Configuration | ~20 | JSON5/TOML/CMake/TS |

## Naming Conventions

- **ArkTS**: PascalCase components (`ChatView`), camelCase methods, `.ets` extension
- **Rust**: snake_case modules/functions, PascalCase types, snake_case files
- **C++**: snake_case files, PascalCase FFI types
- **Config**: `.json5` (HarmonyOS), `.toml` (Rust), `.sh` (scripts)

