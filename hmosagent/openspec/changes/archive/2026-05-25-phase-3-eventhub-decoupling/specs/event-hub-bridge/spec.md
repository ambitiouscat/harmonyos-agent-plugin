## event-hub-bridge

### Purpose
Zero-coupling bidirectional message pipeline between Godot C++ engine and Web frontend, relayed through ArkTS EventHub with string-keyed events (no direct C++↔ArkTS coupling).

### Requirements

- **REQ-EH-001**: Godot C++ side SHALL push messages to `arktsTasks` queue via `send_data_to_browser(who, target, data)` with string `event` and JSON `data` fields
- **REQ-EH-002**: A dedicated C++ thread (`SubDataThread`) SHALL consume the queue and dispatch via `napi_threadsafe_function` to ArkTS main thread
- **REQ-EH-003**: ArkTS side SHALL route C++→Web messages through `EventHub` key `CPP2WEB`, then forward to Web via `MessagePort.postMessageEvent()`
- **REQ-EH-004**: ArkTS side SHALL route Web→C++ messages from `MessagePort.onMessageEvent()` through `EventHub` key `WEB2CPP`, then forward to C++ via `plugin.frontSlot(data)`
- **REQ-EH-005**: The EventHub keys SHALL be exported from HAR `Index.ets` as constants (`CMD_WEB2CPP`, `EVENT_CPP2WEB`) for downstream consumers
- **REQ-EH-006**: Adding new event types (e.g., `node_selected`) SHALL NOT require changes to the bridge interface — only new event handlers in the dispatch switch

### Implementation

- **C++ queue**: `arktsTasks` (std::queue + mutex + condition_variable) in `arkts_tasks.h/cpp`
- **C++ dispatch thread**: `SubDataThread()` in `i3ddll.cpp`
- **ArkTS relay**: `Index.ets` onPageEnd creates WebMessagePorts and registers EventHub listeners
- **Event routing**: `EditorCenterXComponent.ets` cpp2web() dispatches by event type string
