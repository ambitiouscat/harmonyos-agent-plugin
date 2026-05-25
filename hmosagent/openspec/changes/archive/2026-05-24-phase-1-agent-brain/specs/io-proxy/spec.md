## ADDED Requirements

### Requirement: Blocking C→ArkTS IO proxy

The C++ bridge SHALL provide a `post_fn_proxy(const char* url, const char* body) -> char*` callback conforming to the `PostFn` function pointer type. It SHALL bridge from Rust's synchronous C call to ArkTS's asynchronous HTTP via `napi_threadsafe_function` + `std::condition_variable`, blocking the calling thread until a response is delivered or an error occurs.

#### Scenario: Successful HTTP round-trip
- **WHEN** Rust calls `post_fn("https://api.example.com/chat", "{...}")`
- **THEN** the C++ proxy pushes the request to ArkTS via tsfn
- **AND** blocks the calling thread on `g_io_cv.wait()`
- **AND** ArkTS calls `@ohos.net.http` and the response is stored in `g_io_response`
- **AND** `g_io_cv.notify_one()` wakes the blocked thread
- **AND** the response is `malloc`-duplicated and returned to Rust

#### Scenario: IO not initialized
- **WHEN** `post_fn_proxy` is called but `g_io_tsfn` is null
- **THEN** returns `{"status":"error","error":"IO not initialized"}` without blocking

### Requirement: Dual tsfn initialization

The bridge SHALL export `initAgentWithIo(streamCb, ioCb)` which creates two independent `napi_threadsafe_function` handles: one for streaming callbacks (`g_stream_tsfn`) and one for blocking IO (`g_io_tsfn`). SystemCallbacks SHALL be wired with real proxy functions (not nullptr).

#### Scenario: initAgentWithIo succeeds
- **WHEN** ArkTS calls `initAgentWithIo(streamCallback, ioCallback)` with two valid callbacks
- **THEN** both tsfn handles are created
- **AND** `rust_agent_init` is called with SystemCallbacks containing `post_fn_proxy`, `stream_post_fn_proxy`, `free_str_fn_proxy`
- **AND** returns true

### Requirement: Memory lifecycle safety

The `free_str_fn_proxy` SHALL call C `free()` on the pointer. Strings allocated by `post_fn_proxy` via `malloc` MUST be freed only by `free_str_fn_proxy` (called from Rust via `rust_agent_free_str` for host-allocated strings). Rust-allocated strings returned from `rust_agent_call` MUST be freed by Rust's own `rust_agent_free_str`.

#### Scenario: Host string freed correctly
- **WHEN** Rust receives a `char*` from `post_fn_proxy` and later calls `free_str_fn` on it
- **THEN** the memory is deallocated without cross-heap corruption
