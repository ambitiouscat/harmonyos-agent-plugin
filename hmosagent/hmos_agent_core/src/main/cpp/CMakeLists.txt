cmake_minimum_required(VERSION 3.5.0)
project(hmos_agent_core)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})
set(CMAKE_CXX_STANDARD 17)

# Detect target ABI from the compiler target triple
if(CMAKE_CXX_COMPILER_TARGET MATCHES "x86_64.*")
    set(TARGET_ABI "x86_64")
elseif(CMAKE_CXX_COMPILER_TARGET MATCHES "aarch64.*")
    set(TARGET_ABI "arm64-v8a")
else()
    message(WARNING "Unknown compiler target: ${CMAKE_CXX_COMPILER_TARGET}, defaulting to arm64-v8a")
    set(TARGET_ABI "arm64-v8a")
endif()
message(STATUS "HmosAgent: using libs/${TARGET_ABI}/libagent_core.a")

# --- Import pre-built Rust static library ---
add_library(agent_core STATIC IMPORTED)
set_target_properties(agent_core PROPERTIES
    IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/../../../../libs/${TARGET_ABI}/libagent_core.a
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
