## Architecture

### Spinner Status Bar Architecture

```
SettingsPage (phrase style selection)
  │
  ├─ Preferences.put("thinking_phrase_style")
  └─ AppStorage.setOrCreate("thinking_phrase_style")
       │
       ▼
ChatView.startStatusSpinner()
  │
  ├─ loadPhraseStyle() → AppStorage.get("thinking_phrase_style")
  ├─ loader.getSpinnerPhrases(styleKey)
  │    └─ PhraseLoader.categoryMap.get(styleKey)
  │         └─ agent_thinking_phrases.json (categories[])
  ├─ Fisher-Yates shuffle → phraseQueue
  ├─ setInterval(1.5s) → phraseQueue[++index]
  ├─ setInterval(1s) → elapsedSeconds++
  │
  ▼
┌─────────────────────────────────────────┐
│  Hyperspacing…  (1m 43s)               │  ← Status Bar (fixed above InputBar)
└─────────────────────────────────────────┘
```

### Data-Driven Category System

```
agent_thinking_phrases.json (v2.0, single source of truth)
  {
    "categories": [
      { "id": "claude-original", "name": "Claude Code (187)", "phrases": [...], "completions": [...] },
      ...
    ]
  }
       │
       ├── PhraseLoader.getCategories() → CategoryInfo[]
       │    └── SettingsPage.phraseCategories (replaces hardcoded array)
       │
       ├── PhraseLoader.getSpinnerPhrases(id) → string[]
       │    └── ChatView.status bar (Fisher-Yates + timer)
       │
       └── PhraseLoader.getRandomCompletion(id) → string
            └── ThinkingPanel (completion word on finish)
```

## Key Design Decisions

### 1. 60fps (16ms) Rust-Side Throttling Only

**Decision**: Abandoned 30fps (33ms) MarkdownView setTimeout throttling.

**Why**: Phase 2 design explicitly rejected TS-side double-buffering. Rust's `FrameThrottler` at 16ms already produces frame-aligned output. Adding TS throttle creates 49ms effective latency (~20fps).

**Fix path**: Root cause of stutter was `ForEach` full repaint, not data frequency. Solution: restrict `@Trace` to the last active message's text span, avoiding tree-wide re-render.

### 2. Fisher-Yates Shuffle with Boundary Guard

**Decision**: Shuffle once per queue cycle, guard against consecutive duplicate.

**Why**: Pure random (Math.random each tick) allows repeats. Fisher-Yates guarantees uniform distribution. When new queue's [0] equals previous cycle's last element, swap with [1] — safe for arrays of any length ≥ 2.

### 3. Status Bar in ChatView, Not MessageBubble

**Decision**: Move spinner from message-internal `ThinkingPanel` to fixed `Row` above `InputBar` in `ChatView`.

**Why**: MessageBubble scrolls away. Claude Code UX shows fixed status line above input. This is more visible and doesn't disappear during scroll.

### 4. Completion Words Inlined per Category

**Decision**: Remove `SPINNER_TO_COMPLETION` mapping table. Each category in JSON carries its own `completions` array.

**Why**: JSON is single source of truth. Adding/removing categories requires zero code changes.

### 5. AboutToDisappear Full Cleanup (i3d544)

**Decision**: `EditorCenterXComponent.aboutToDisappear()` must unregister all 3 EventHub events + call `plugin.threadSafeDataCase(null)`.

**Why**: The `cpp2web` NAPI callback holds a strong reference to `this`. Without explicit cleanup, the UI instance is retained across panel open/close cycles, causing JS heap growth.

## Risk / Trade-offs

- **[Trade-off] JSON file size**: Inlining completions adds ~200 bytes per category. Acceptable for 11 categories.
- **[Risk] PhraseLoader load timing**: `load()` is async (rawfile read). Called in `ChatView.aboutToAppear()` — may briefly show fallback words on very first frame. Acceptable: rawfile reads complete in <10ms.
- **[Risk] setInterval in ArkTS**: Two timers per streaming session. Cleaned up in `stopStatusSpinner()` on stream end/error/abort. No known leaks.
