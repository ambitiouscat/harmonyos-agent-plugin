## Context

Phase 0-1 built the Rust brain and C ABI. Phase 2 builds the face. The UI must render multi-part streamed messages (text, reasoning, tool calls, tool results) without WebView, at 60fps minimum, with persistent settings.

## Goals / Non-Goals

**Goals:**
- Structured JSON delta protocol replacing raw char streaming
- ArkUI V2 reactive state (`@ObservedV2`, `@Trace`) for fine-grained re-render
- Native Markdown rendering (no WebView) for 4 block types
- Collapsible thinking panel and tool call cards
- Preferences-backed settings page

**Non-Goals:**
- Abort streaming at native level (UI button only in Phase 2)
- Full CommonMark parser (4 block types only)
- Dark/light theme toggle
- Conversation list / multi-session

## Decisions

### 1. Structured JSON Delta over Raw Characters

**Choice**: Rust emits ContentPart JSON strings (`{"type":"text","text":"H"}`) via the existing stream callback.

**Why**: TS side can `JSON.parse` and route to the correct part type without fuzzy text classification. The stream callback's `(char*, uint8_t)` signature is unchanged — only the *content* of the char* changes.

**Alternative**: Binary protobuf — rejected for Phase 2 complexity. JSON is human-debuggable.

### 2. Single-Layer Throttling (Rust Only)

**Choice**: Rust `FrameThrottler` (16ms window) does all throttling. TS side applies deltas directly to `@Trace` properties without additional timers.

**Why**: ArkUI V2's virtual machine already merges multiple `@Trace` writes within one frame into a single commit. Adding a TS-side debouncer would create 32ms double-buffering.

### 3. @State in V1 Entry over @Local in V2 ChatView

**Choice**: Entry Index.ets uses `@State messages: ArkMessage[]` (V1 compatible). ChatView.ets uses `@Local` internally but the actual streaming state lives in Index.

**Why**: `@Entry` components must use V1 `@Component`. V2 child components rendered inside a V1 parent lose independent reactivity. Centralizing state at the Entry level guarantees V1 change detection triggers UI refresh.

### 4. No WebView (RichText) for Markdown

**Choice**: Pure ArkUI `Text`/`Span`/`Column`/`Row` composition via `MarkdownParser.ts` → `MarkdownView.ets`.

**Why**: `RichText` is a WebView under the hood and causes memory spikes and ANR in scrolling lists. Native ArkUI components are GPU-accelerated and lightweight.

### 5. Preferences `configure` Route

**Choice**: Settings are injected via `rust_agent_call("configure", jsonConfig)` using the existing JSON router.

**Why**: No new FFI surface needed. Rust stores config in `RwLock<Value>`. Cold-start: ArkTS reads preferences → calls configure → Rust core has API keys ready before first HTTP call.

## Risks / Trade-offs

- **[Risk] V1/V2 component compatibility** → Mitigated by centralizing state in Index (V1). V2 child components are pure rendering.
- **[Risk] Markdown parser scope** → Only 4 block types. Complex formatting silently degrades to paragraph text. Acceptable for Phase 2.
- **[Trade-off] ChatView not used at Entry level** → ChatView is built but Index uses inline rendering for V1 compatibility. ChatView becomes the primary component in Phase 3 when Entry upgrades to V2.
