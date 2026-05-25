## log-filter

### Purpose
Regex-based noise stripping and deduplication for Godot engine console output flowing through the `data_transform` event pipeline, preventing Vulkan/texture/frame-queue spam from flooding the Web UI and VibeCoding chat panel.

### Requirements

- **REQ-LF-001**: The filter SHALL suppress lines matching any of a configurable set of regex patterns (Vulkan rendering, texture loading, frame queue, shader compile, RID allocation)
- **REQ-LF-002**: The filter SHALL maintain a sliding window (default size 20) of recent messages for deduplication
- **REQ-LF-003**: Repeated identical messages SHALL be suppressed, emitting `[Repeated N times] <message>` only every 10th occurrence
- **REQ-LF-004**: Non-matching messages SHALL pass through unchanged
- **REQ-LF-005**: The filter SHALL be instantiated at the host level (`hap_editor`), not inside the HAR, to keep the HAR portable

### Implementation

- `LogFilter.ts` class with `filter(data: string): string | null` method
- Intake: raw Godot console line
- Return: filtered string or `null` (drop entirely)
- Used in `EditorCenterXComponent.ets` cpp2web() `data_transform` branch
