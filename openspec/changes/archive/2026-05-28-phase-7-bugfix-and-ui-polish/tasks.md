## 1. Agent Loop HTTP 400 Fixes (R4-R5)

- [x] 1.1 Fix HTTP error handling: replace `send_json()` with Agent using `http_status_as_error(false)`, manually read response body on 4xx/5xx
- [x] 1.2 Remove `tool_choice: "auto"` from API request body to maximize provider compatibility
- [x] 1.3 Remove `content: null` from assistant messages with only tool_calls
- [x] 1.4 Add `normalize_args()` helper — replace null/non-object tool arguments with `{}`
- [x] 1.5 Capture `delta.reasoning_content` in SSE parser and echo back as separate field in `messages_to_api`
- [x] 1.6 Add `LlmResponse.reasoning_text` field and propagate through `agent_loop_run`

## 2. Claude API TODO

- [x] 2.1 Add TODO comment in `loop_engine.rs` for Claude native output format support
- [x] 2.2 Create Trilium TODO note under project notes

## 3. Status Bar Refactor

- [x] 3.1 Create `StatusBar.ets` — dynamic status items with FlexWrap, named `StatusItem` interface
- [x] 3.2 Create `InfoBar.ets` — fixed info line (model, folder, context, memory) above input bar
- [x] 3.3 Update `ChatView.ets` — replace old status row with StatusBar + InfoBar, add `upsertStatusItem` / `removeStatusItem` methods
- [x] 3.4 Add `updateContextUsage()` — estimate token count from message history
- [x] 3.5 Read model context window from ProviderLoader `limit.context` metadata

## 4. Tool Colors & Theme

- [x] 4.1 Add `tool_colors` object to all 4 themes in `themes.json` with per-tool hex colors
- [x] 4.2 Add `tool_colors: Record<string, string>` to `ThemeConfig` interface and parse in `ThemeLoader`
- [x] 4.3 Add `getToolColor()` to `ChatView` and pass to `StatusBar` items
- [x] 4.4 Change default theme from dark to light (`currentId` and `DEFAULT_THEME`)
- [x] 4.5 Persist theme selection to preferences (`theme_color` key), restore on cold start via async `restoreFromPrefs()`
