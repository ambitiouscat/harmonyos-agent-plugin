#include "napi/native_api.h"
#include "hilog/log.h"
#include "agent_core.h"
#include <string>
#include <cstring>
#include <thread>

#undef LOG_DOMAIN
#undef LOG_TAG
#define LOG_DOMAIN 0x3200
#define LOG_TAG "HMOS_AGENT_BRIDGE"

// ---- Thread-Safe Function for Streaming Callbacks ----
static napi_threadsafe_function g_stream_tsfn = nullptr;

// Data payload passed across the thread boundary from Rust to JS main thread.
struct ChunkPayload {
    std::string data;
    uint8_t event_type;  // 0 = data chunk, 1 = done, 2 = error
};

// Main-thread callback: invoked by the NAPI runtime to deliver a chunk to JS.
static void DeliverChunkToJS(napi_env env, napi_value js_callback,
                              void* /*context*/, void* data) {
    ChunkPayload* payload = static_cast<ChunkPayload*>(data);
    if (payload == nullptr) return;

    napi_value args[2];
    napi_create_string_utf8(env, payload->data.c_str(), NAPI_AUTO_LENGTH, &args[0]);
    napi_create_uint32(env, payload->event_type, &args[1]);

    napi_value result;
    napi_status status = napi_call_function(env, nullptr, js_callback, 2, args, &result);
    if (status != napi_ok) {
        OH_LOG_ERROR(LOG_APP, "DeliverChunkToJS: napi_call_function failed %{public}d", status);
    }

    delete payload;
}

// ---- OnChunkBridge: C callback invoked from Rust threads ----
// Forwards every chunk through the thread-safe function to the JS main thread.
static void OnChunkBridge(const char* chunk_data, uint8_t event_type) {
    if (g_stream_tsfn == nullptr) {
        OH_LOG_WARN(LOG_APP, "OnChunkBridge called but tsfn is null");
        return;
    }

    ChunkPayload* payload = new ChunkPayload{
        .data = chunk_data ? std::string(chunk_data) : "",
        .event_type = event_type,
    };

    napi_status status = napi_call_threadsafe_function(
        g_stream_tsfn, payload, napi_tsfn_blocking);
    if (status != napi_ok) {
        OH_LOG_ERROR(LOG_APP, "napi_call_threadsafe_function failed %{public}d", status);
        delete payload;
    }
}

// ---- Safe JS string to C++ string conversion ----
// Uses the two-call pattern to avoid off-by-one buffer overflows.
static std::string JsStringToCpp(napi_env env, napi_value js_val) {
    size_t byte_size = 0;
    napi_get_value_string_utf8(env, js_val, nullptr, 0, &byte_size);

    char* buf = new char[byte_size + 1];
    napi_get_value_string_utf8(env, js_val, buf, byte_size + 1, &byte_size);
    std::string result(buf, byte_size);
    delete[] buf;
    return result;
}

// ---- NAPI: initAgent(jsCallback: function): boolean ----
static napi_value InitAgent(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);

    if (argc < 1) {
        OH_LOG_ERROR(LOG_APP, "initAgent: callback argument required");
        napi_value result;
        napi_get_boolean(env, false, &result);
        return result;
    }

    napi_value tsfn_name;
    napi_create_string_utf8(env, "AgentStreamCB", NAPI_AUTO_LENGTH, &tsfn_name);

    napi_status status = napi_create_threadsafe_function(
        env,
        args[0],
        nullptr,
        tsfn_name,
        0,                 // max_queue_size = 0 (unlimited)
        1,                 // initial thread count
        nullptr,           // thread finalizer data
        nullptr,           // thread finalizer callback
        nullptr,           // context
        DeliverChunkToJS,
        &g_stream_tsfn
    );

    if (status != napi_ok) {
        OH_LOG_ERROR(LOG_APP, "napi_create_threadsafe_function failed %{public}d", status);
        napi_value result;
        napi_get_boolean(env, false, &result);
        return result;
    }

    // Register OnChunkBridge as the streaming callback in Rust.
    rust_agent_register_stream_cb(OnChunkBridge);

    // Build SystemCallbacks. Phase 0: no real HTTP — all nullptrs.
    SystemCallbacks cbs;
    cbs.post_fn = nullptr;
    cbs.stream_post_fn = nullptr;
    cbs.free_str_fn = nullptr;

    bool ok = rust_agent_init(cbs);

    OH_LOG_INFO(LOG_APP, "initAgent: rust_agent_init returned %{public}d", ok);

    napi_value result;
    napi_get_boolean(env, ok, &result);
    return result;
}

// ---- NAPI: agentCall(action: string, args: string): string ----
static napi_value AgentCall(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);

    std::string action = JsStringToCpp(env, args[0]);
    std::string json_args = JsStringToCpp(env, args[1]);

    char* rust_res = rust_agent_call(action.c_str(), json_args.c_str());

    napi_value js_res;
    napi_create_string_utf8(env, rust_res, NAPI_AUTO_LENGTH, &js_res);
    rust_agent_free_str(rust_res);

    return js_res;
}

// ---- NAPI: testNetwork(): boolean ----
static napi_value TestNetwork(napi_env env, napi_callback_info /*info*/) {
    bool ok = test_network();
    OH_LOG_INFO(LOG_APP, "testNetwork: %{public}d", ok);
    napi_value result;
    napi_get_boolean(env, ok, &result);
    return result;
}

// ---- NAPI: testFile(dir: string): boolean ----
static napi_value TestFile(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);

    std::string dir = JsStringToCpp(env, args[0]);
    bool ok = test_file(dir.c_str());
    OH_LOG_INFO(LOG_APP, "testFile: %{public}d path=%{public}s", ok, dir.c_str());
    napi_value result;
    napi_get_boolean(env, ok, &result);
    return result;
}

// ---- Module Registration ----
EXTERN_C_START
static napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor desc[] = {
        {"initAgent",   nullptr, InitAgent,   nullptr, nullptr, nullptr, napi_default, nullptr},
        {"agentCall",   nullptr, AgentCall,   nullptr, nullptr, nullptr, napi_default, nullptr},
        {"testNetwork", nullptr, TestNetwork, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"testFile",    nullptr, TestFile,    nullptr, nullptr, nullptr, napi_default, nullptr},
    };
    napi_define_properties(env, exports, sizeof(desc) / sizeof(desc[0]), desc);
    return exports;
}
EXTERN_C_END

static napi_module demoModule = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = nullptr,
    .nm_register_func = Init,
    .nm_modname = "native_bridge",
    .nm_priv = nullptr,
    .reserved = {},
};

extern "C" __attribute__((constructor)) void RegisterNativeBridgeModule(void) {
    napi_module_register(&demoModule);
}
