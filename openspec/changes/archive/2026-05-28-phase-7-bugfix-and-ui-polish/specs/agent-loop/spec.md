## ADDED Requirements

### Requirement: HTTP error response body capture
The system SHALL capture and display the API error response body when the LLM API returns a 4xx/5xx status code, instead of a generic error message.

#### Scenario: API returns 400 with error detail
- **WHEN** the LLM API returns HTTP 400 with a JSON error body
- **THEN** the error message displayed to the user SHALL contain the API's `error.message` field
- **AND** the full error detail SHALL be passed through the stream callback (eventType 2)

#### Scenario: API returns 400 without JSON body
- **WHEN** the LLM API returns HTTP 400 with a non-JSON response body
- **THEN** the error message SHALL contain the HTTP status code and first 300 characters of the response body

### Requirement: reasoning_content echo-back
The system SHALL capture `reasoning_content` from SSE delta chunks and SHALL include it as a separate `reasoning_content` field in subsequent conversation turns.

#### Scenario: DeepSeek R1 thinking mode
- **WHEN** the API streams a delta containing `reasoning_content`
- **THEN** the content SHALL be accumulated in the assistant message
- **AND** in the next API request, the assistant message SHALL include `reasoning_content` as a top-level field distinct from `content`

### Requirement: No tool_choice parameter
The system SHALL NOT include a `tool_choice` parameter in API requests. Tool selection SHALL rely on the API default behavior (equivalent to `"auto"`).

### Requirement: Null-safe tool arguments
The system SHALL normalize tool call arguments: null or non-object values SHALL be replaced with an empty object `{}` before serialization.

### Requirement: No content null in assistant tool messages
The system SHALL omit the `content` field from assistant messages that contain only tool calls (no text), rather than setting it to `null`.
