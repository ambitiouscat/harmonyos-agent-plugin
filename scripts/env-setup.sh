#!/usr/bin/env bash
# Source this script before cross-compiling or building HAP.
# Usage: source scripts/env-setup.sh
# Works from Git Bash (Windows) or WSL.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_ROOT="$PROJECT_ROOT/rust_core"
HMOS_ROOT="$PROJECT_ROOT/hmosagent"
LIBS_DIR="$HMOS_ROOT/libs"

# DevEco paths: always use D:/ format (works in Git Bash AND WSL)
DEVECO_HOME="D:/Program Files/Huawei/DevEco Studio"
DEVECO_SDK_HOME="$DEVECO_HOME/sdk"
DEVECO_NODE="$DEVECO_HOME/tools/node/node.exe"
DEVECO_HVIGOR="$DEVECO_HOME/tools/hvigor/bin/hvigorw.js"
NDK_BASE="D:/Program Files/Huawei/DevEco Studio/sdk/default/openharmony/native"
NDK_BIN="$NDK_BASE/llvm/bin"
NDK_SYSROOT="$NDK_BASE/sysroot"

export DEVECO_SDK_HOME
export PATH="$NDK_BIN:$PATH"

echo "[env] PROJECT_ROOT=$PROJECT_ROOT"
echo "[env] HMOS_ROOT=$HMOS_ROOT"
echo "[env] LIBS_DIR=$LIBS_DIR"
echo "[env] NDK_BIN=$NDK_BIN"
