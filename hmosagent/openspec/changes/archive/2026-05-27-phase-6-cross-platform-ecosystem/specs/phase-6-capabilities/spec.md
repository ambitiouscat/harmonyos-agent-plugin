# Phase 6 Capabilities

## Session Manager

- **REQ-SESSION-001**: CREATE session with auto-generated ID + timestamp title
- **REQ-SESSION-002**: LIST all sessions from `{filesDir}/sessions/index.json`
- **REQ-SESSION-003**: LOAD session by ID, restore full message history
- **REQ-SESSION-004**: DELETE session by ID, remove from index + file
- **REQ-SESSION-005**: SAVE session with current messages as JSON
- **REQ-SESSION-006**: INIT with host `filesDir` path
- **REQ-SESSION-007**: Lazy-create: no disk write until first message sent

## Skills Registry

- **REQ-SKILLS-001**: REGISTER builtin commands (/search, /scan, /goal, /file, /session)
- **REQ-SKILLS-002**: LIST all registered skills via `get_registered_skills`
- **REQ-SKILLS-003**: FUZZY match by name or description prefix

## Conversation Drawer

- **REQ-DRAWER-001**: SLIDE in from left edge, 80% width, dark panel
- **REQ-DRAWER-002**: TOGGLE via hamburger button, CLOSE via ✕ or backdrop click
- **REQ-DRAWER-003**: Conditionally rendered (unmounted when closed, no event interception)
- **REQ-DRAWER-004**: Asymmetric transition animation (entry 220ms, exit 180ms)

## Command Palette

- **REQ-PALETTE-001**: POPUP on `/` input, show matching commands
- **REQ-PALETTE-002**: DYNAMIC fetch from FFI `get_registered_skills`
- **REQ-PALETTE-003**: CLICK to autocomplete command name in input

## Theme System

- **REQ-THEME-001**: MULTI-THEME via JSON config (Dark, Light, Sepia, Solarized)
- **REQ-THEME-002**: CYCLE themes via toolbar button
- **REQ-THEME-003**: ALL components respond to theme change (ChatView, MarkdownView, InputBar)

## SSE Reconnect

- **REQ-RECONNECT-001**: RETRY HTTP SSE request on connection failure (up to 3 attempts)
- **REQ-RECONNECT-002**: EXPONENTIAL backoff (500ms base, 5s cap)
- **REQ-RECONNECT-003**: DEDUP overlapping content across reconnections
- **REQ-RECONNECT-004**: NOTIFY UI of reconnection status via delta type
