# Spec: Thinking Phrases (趣味思考短语系统)

## Overview

Agent 在等待 LLM 响应期间，在输入框上方显示动态轮播的趣味短语状态栏。短语分类由 JSON 配置文件定义，用户可在 Settings 中选择。

## Requirements

### REQ-TP-001: Phrase Data Loading

The system SHALL load phrase categories from a single JSON configuration file (`agent_thinking_phrases.json`) at runtime.

**Acceptance:**
- `PhraseLoader.load()` reads rawfile and parses JSON within 10ms
- On load failure, falls back to hardcoded English fallback words (`["Thinking", "Processing", ...]`)
- On missing category key, falls back to `claude-original` category

### REQ-TP-002: Dynamic Category List

Settings page SHALL display available phrase categories dynamically from the loaded JSON data, NOT from a hardcoded array.

**Acceptance:**
- `PhraseLoader.getCategories()` returns all categories from JSON
- SettingsPage renders dropdown from `getCategories()` result
- Adding a category to JSON requires zero code changes

### REQ-TP-003: Category Self-Containment

Each category in the JSON SHALL contain its own `phrases` array and `completions` array. No external mapping tables.

**Acceptance:**
- New category: `{ "id": "...", "name": "...", "phrases": [...], "completions": [...] }`
- No code changes needed to wire up completions for a new category

### REQ-TP-004: Fisher-Yates Shuffle Rotation

When streaming starts, the status bar SHALL display phrases from the selected category in rotation, shuffled via Fisher-Yates algorithm.

**Acceptance:**
- Phrases cycle every 1.5 seconds
- No consecutive duplicate phrases (boundary guard: swap [0]↔[1] if needed)
- When queue exhausts, re-shuffle and continue

### REQ-TP-005: Elapsed Timer

The status bar SHALL display elapsed time since streaming started, updating every 1 second.

**Acceptance:**
- Format: `Phrase… (Xs)` for under 60s, `Phrase… (Xm Ys)` for over 60s
- Timer starts when `startStatusSpinner()` is called
- Timer stops when streaming ends, error occurs, or user aborts

### REQ-TP-006: Status Bar Position

The status bar SHALL appear as a fixed row between the message List and the InputBar in ChatView. It SHALL NOT be inside a scrollable message bubble.

**Acceptance:**
- Status bar visible when `isStreaming === true`
- Positioned above InputBar, below message List
- Disappears immediately when streaming stops

### REQ-TP-007: Completion Word Display

When thinking completes normally (stream ends, content exists), the system SHALL display a random completion word from the selected category's `completions` array.

**Acceptance:**
- Completion word shown for 1 second, then fades out
- Completion keyed to the same style ID as the phrases
- On abort (user clicks stop), no completion word shown

### REQ-TP-008: Settings Persistence

The selected phrase style SHALL be persisted to Preferences and broadcast via AppStorage.

**Acceptance:**
- `Preferences.put('thinking_phrase_style', styleId)` on save
- `AppStorage.setOrCreate('thinking_phrase_style', styleId)` for runtime components
- `ChatView.loadPhraseStyle()` reads from AppStorage before each `startStatusSpinner()`

---

## Verification

| Requirement | Test Method |
|------------|-------------|
| REQ-TP-001 | Verify 11 categories load; delete JSON → fallback words appear |
| REQ-TP-002 | Add "test-cat" to JSON → appears in SettingsPage without code change |
| REQ-TP-003 | Verify no `SPINNER_TO_COMPLETION` exists in PhraseLoader source |
| REQ-TP-004 | Send 10 messages → verify no consecutive duplicate in any session |
| REQ-TP-005 | Send message → observe timer increments from 0s upward |
| REQ-TP-006 | Verify status bar does not scroll with messages |
| REQ-TP-007 | Let stream complete → verify completion word fades after 1s |
| REQ-TP-008 | Switch style in Settings → send message → verify new style's phrases appear |
