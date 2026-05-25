## ADDED Requirements

### Requirement: Structured JSON delta format

The stream callback SHALL deliver ContentPart JSON strings instead of raw characters. Each delta SHALL be a valid JSON object with a `"type"` field matching ContentPart variants (`text`, `reasoning`, `tool_call`, `tool_result`).

#### Scenario: Text delta
- **WHEN** Rust emits `{"type":"text","text":"Hello"}`
- **THEN** ArkTS parses and appends "Hello" to the current text part

#### Scenario: Reasoning delta
- **WHEN** Rust emits `{"type":"reasoning","text":"thinking...","collapsed":false}`
- **THEN** ArkTS creates or updates a reasoning part with collapsed=false

#### Scenario: Tool call delta
- **WHEN** Rust emits `{"type":"tool_call","id":"tc1","name":"search","arguments":"{}"}`
- **THEN** ArkTS creates a tool_call part with toolId and name

#### Scenario: Tool result delta
- **WHEN** Rust emits `{"type":"tool_result","id":"tc1","name":"search","output":"found","is_error":false}`
- **THEN** ArkTS creates a tool_result part with output and isError flag

### Requirement: Simulation maintains backward compatibility

`start_stream_sim` SHALL emit the new JSON format. Existing Phase 0 tests SHALL pass because the data format change is at the content level, not the C ABI.

#### Scenario: Sim emits JSON
- **WHEN** `test_stream` action is triggered with chunks=5
- **THEN** 5 structured JSON deltas are pushed via the stream callback
