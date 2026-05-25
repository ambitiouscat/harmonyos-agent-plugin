## ADDED Requirements

### Requirement: Reactive data models

`ArkMessage` and `ArkContentPart` SHALL use `@ObservedV2` and `@Trace` decorators for fine-grained UI reactivity. `ArkContentPart.parts` array mutations and individual property updates SHALL trigger only the affected component subtree.

#### Scenario: Text append triggers re-render
- **WHEN** `part.text += "hello"` is applied to an existing ContentPart
- **THEN** only the Text component bound to that part re-renders

### Requirement: MarkdownParser — 4 block types

`MarkdownParser.parse()` SHALL recognize heading (`#`), paragraph, fenced code block (`` ``` ``), and bullet list (`- `). Unrecognized lines SHALL become paragraphs.

#### Scenario: Code block with language
- **WHEN** input contains `` ```rust\nfn main() {}\n``` ``
- **THEN** a code_block block is produced with language="rust" and text="fn main() {}"

### Requirement: MarkdownView — native ArkUI rendering

`MarkdownView` SHALL render parsed blocks using only `Text`, `Span`, `Flex`, `Column`, `Row`. It SHALL NOT use `RichText` (WebView). Code blocks SHALL have a "Copy" button using `@ohos.pasteboard`.

#### Scenario: Code block copy
- **WHEN** user taps "Copy" on a code block
- **THEN** the code text is placed on the system clipboard

### Requirement: ThinkingPanel

`ThinkingPanel` SHALL display reasoning text with a glassmorphism background, a grey left accent bar, and a collapse/expand toggle bound to `@Trace collapsed`.

#### Scenario: Collapse toggle
- **WHEN** user taps the collapse arrow
- **THEN** reasoning text hides/shows with animation

### Requirement: ToolCard

`ToolCard` SHALL display tool name, colored status dot (orange=running, green=success, red=error), and collapsible output text. Long output SHALL be truncated at 10 lines with ellipsis.

#### Scenario: Error tool result
- **WHEN** isError=true and output is non-empty
- **THEN** the status dot is red

### Requirement: MessageBubble router

`MessageBubble` SHALL iterate `message.parts` and route each to the appropriate sub-component: text → MarkdownView, reasoning → ThinkingPanel, tool_call → ToolCard, tool_result → ToolCard.

### Requirement: InputBar send/stop toggle

`InputBar` SHALL show a blue send button when idle and a red stop button when `isStreaming=true`.

#### Scenario: Stop button appears during streaming
- **WHEN** isStreaming transitions to true
- **THEN** the button changes from "↑" (blue) to "■" (red)

### Requirement: ChatView with applyDelta

`ChatView` SHALL expose `applyDelta(jsonDelta: string)` which parses the JSON and merges it into the current assistant message's parts array, creating a new message if needed.

#### Scenario: Sequential text deltas merge
- **WHEN** two text deltas arrive for the same message
- **THEN** they append to the same text part, not create duplicates
