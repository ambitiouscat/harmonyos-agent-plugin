cmake_minimum_required(VERSION 3.5.0)
project(hmos_agent_core)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})
set(CMAKE_CXX_STANDARD 17)

# --- Import pre-built Rust static library ---
# libagent_core.a must be copied to hmosagent/libs/arm64-v8a/ before building
add_library(agent_core STATIC IMPORTED)
set_target_properties(agent_core PROPERTIES
    IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/../../../../libs/arm64-v8a/libagent_core.a
)

# --- Build NAPI bridge shared library ---
add_library(native_bridge SHARED
    native_bridge.cpp
    musl_shim.c
)

target_include_directories(native_bridge PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

find_library(ace_napi_lib ace_napi.z)
find_library(hilog_lib hilog_ndk.z)

target_link_libraries(native_bridge PUBLIC
    ${ace_napi_lib}
    ${hilog_lib}
    agent_core
    libc++.so
)
