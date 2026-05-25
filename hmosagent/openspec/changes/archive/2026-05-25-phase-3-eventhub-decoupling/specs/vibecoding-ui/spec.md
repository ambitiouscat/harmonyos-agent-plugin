## vibecoding-ui

### Purpose
Toggleable split-pane layout embedding the HmosAgent ChatView alongside the 3D editor viewport, enabling AI-assisted coding without disrupting normal editor workflow.

### Requirements

- **REQ-VU-001**: A top toggle bar (28px, position absolute/overlay) SHALL offer Normal and VibeCoding mode buttons
- **REQ-VU-002**: Normal mode: Stack width 100% (Web + XComponent fills entire space)
- **REQ-VU-003**: VibeCoding mode: Row layout with Web at 65% width + ChatView at 35% width, white background, left border separator
- **REQ-VU-004**: Toggling modes SHALL NOT reload the Web view or reinitialize ChatView — only change container widths
- **REQ-VU-005**: The toggle bar SHALL be hidden during initial loading (`if (!this.loading)`)
- **REQ-VU-006**: ChatView SHALL be imported from the `hmos_agent_core` HAR binary, not from source

### Implementation

- `hap_editor/Index.ets`: `@State isVibeMode: boolean`, `if (this.isVibeMode)` conditional rendering
- `hap_editor/oh-package.json5`: `"hmos_agent_core": "file:./libs/hmos_agent_core.har"`
- XComponent and Web embed lifecycle unchanged from Normal mode
