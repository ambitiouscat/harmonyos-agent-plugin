## ADDED Requirements

### Requirement: Per-tool color configuration
The system SHALL support per-tool colors configured in `themes.json` under a `tool_colors` key. Each theme SHALL define a mapping from tool name to hex color string.

#### Scenario: Tool color applied
- **WHEN** a tool status item is displayed in the status bar
- **THEN** the dot and text SHALL use the color specified in the active theme's `tool_colors` for that tool name
- **AND** the background SHALL use the same color at reduced opacity

#### Scenario: Fallback color for unknown tools
- **WHEN** a tool name is not found in the active theme's `tool_colors`
- **THEN** the system SHALL use the theme's `accent` color as fallback

#### Scenario: Theme switch updates tool colors
- **WHEN** the user changes the active theme
- **THEN** all tool status items SHALL immediately reflect the new theme's color palette
