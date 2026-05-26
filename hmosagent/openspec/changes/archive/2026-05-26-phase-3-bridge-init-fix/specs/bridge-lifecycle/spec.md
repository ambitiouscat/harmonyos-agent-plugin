## bridge-lifecycle

### Purpose
Ensure RustAgentBridge singleton survives multiple initializations from different call sites without losing native state or producing false-negative ready status.

### Requirements

- **REQ-BL-001**: `init()` SHALL return `true` immediately if `this.initialized` is already `true`, only updating `streamCallback`
- **REQ-BL-002**: `initWithIo()` SHALL return `true` immediately if `this.initialized` is already `true`, only updating `streamCallback` and `ioCallback`
- **REQ-BL-003**: `initBridge()` SHALL return without action if `this.bridgeReady` is already `true`
- **REQ-BL-004**: Neither `init()` nor `initWithIo()` SHALL call any native method when already initialized
- **REQ-BL-005**: `handleSend()` SHALL display an error message in the chat when `bridge.call()` returns an error status or throws an exception
- **REQ-BL-006**: Native bridge (C++) SHALL NOT be modified — all guards are in ArkTS layer
