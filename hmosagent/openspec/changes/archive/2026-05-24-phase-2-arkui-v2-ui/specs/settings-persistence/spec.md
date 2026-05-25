## ADDED Requirements

### Requirement: SettingsPage with Preferences

`SettingsPage` SHALL provide UI controls for API Key (password-masked), Base URL, Temperature (0.0-2.0 slider), and Max Tokens (256-32768 slider). Values SHALL be persisted to `@ohos.data.preferences` under the store name `hmos_agent_config`.

#### Scenario: Save and reload
- **WHEN** user modifies settings and saves
- **THEN** preferences are flushed to disk
- **AND** on cold restart, values are restored without re-entry

### Requirement: configure route injection

Settings SHALL be injected into Rust core via `rust_agent_call("configure", configJson)`. The Rust side SHALL store the config in a global `RwLock<Value>` accessible to HTTP clients.

#### Scenario: Configure on cold start
- **WHEN** app starts and preferences exist
- **THEN** `rust_agent_call("configure", "{...}")` is called once
- **AND** subsequent HTTP requests use the injected API key and base URL

### Requirement: Configure JSON schema

The config JSON SHALL use the following keys: `api_key` (string), `base_url` (string), `temperature` (float), `max_tokens` (integer).

#### Scenario: Valid config
- **WHEN** `{"api_key":"sk-...","base_url":"https://api.siliconflow.cn","temperature":0.7,"max_tokens":4096}` is sent
- **THEN** Rust returns `{"status":"ok","message":"Configuration stored"}`
