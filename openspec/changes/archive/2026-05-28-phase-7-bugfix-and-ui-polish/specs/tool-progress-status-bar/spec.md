## ADDED Requirements

### Requirement: Dynamic status items
The system SHALL support a dynamic status bar (StatusBar) that displays transient status items in a wrapping Flex layout. Items SHALL support insertion, update, and removal at runtime.

#### Scenario: Fun phrase always first
- **WHEN** streaming starts and a thinking phrase is added (id='phrase')
- **THEN** the phrase item SHALL appear as the first element in the status bar
- **AND** subsequent status items (tools, etc.) SHALL appear after it

#### Scenario: Tool call status appears
- **WHEN** the agent loop starts executing a tool (eventType 3 / tool_start)
- **THEN** a status item with the tool name and tool-specific color SHALL appear in the status bar

#### Scenario: Tool call status removed on completion
- **WHEN** the agent loop finishes (eventType 1 or 2)
- **THEN** tool status items (id='tool') SHALL be removed
- **AND** remaining items SHALL shift left to fill the gap

#### Scenario: Auto-wrap to multiple lines
- **WHEN** status items exceed one row of screen width
- **THEN** items SHALL wrap to a second row automatically

### Requirement: Fixed info bar
The system SHALL display a fixed information bar (InfoBar) immediately above the input field, showing model name, current folder, context window usage, and optionally memory usage.

#### Scenario: Model name display
- **WHEN** the API configuration is applied with a model name
- **THEN** the model name SHALL appear in the info bar with an 'M' prefix

#### Scenario: Context usage display
- **WHEN** the context usage is updated after a conversation turn
- **THEN** the info bar SHALL display current token usage and context window capacity (e.g., "185k/128k")

#### Scenario: Context window from model metadata
- **WHEN** the provider is selected and model metadata contains `limit.context`
- **THEN** the info bar SHALL use that value as the context window capacity
- **WHEN** model metadata is unavailable
- **THEN** the info bar SHALL display a fallback value
