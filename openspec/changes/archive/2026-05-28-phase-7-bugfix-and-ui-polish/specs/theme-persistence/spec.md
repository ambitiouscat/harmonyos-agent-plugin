## ADDED Requirements

### Requirement: Theme persistence across restarts
The system SHALL persist the user's selected theme to device preferences storage, and SHALL restore the saved theme on cold start.

#### Scenario: Theme saved on change
- **WHEN** the user cycles to a different theme
- **THEN** the theme selection SHALL be saved to preferences with key `theme_color`
- **AND** the change SHALL take effect immediately in the current session via AppStorage

#### Scenario: Theme restored on cold start
- **WHEN** the app starts with no in-memory theme state
- **THEN** the system SHALL asynchronously read the saved theme from preferences
- **AND** apply it as the active theme

#### Scenario: Default theme
- **WHEN** no theme preference has been saved (first launch)
- **THEN** the system SHALL default to the Light theme
