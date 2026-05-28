#!/usr/bin/env bash
# Source this script before building HAR/HAP from WSL2.
# Usage: source scripts/env-setup-wsl.sh
# WSL2 version: node.exe runs from /mnt/d/... but needs Windows-format args.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_ROOT="$PROJECT_ROOT/rust_core"
HMOS_ROOT="$PROJECT_ROOT/hmosagent"
LIBS_DIR="$HMOS_ROOT/libs"

# DevEco paths for WSL2:
# - executables: /mnt/d/... (WSL can launch Windows .exe this way)
# - args to Windows exes: D:/... (Windows programs need Windows paths)
DEVECO_WIN="D:/Program Files/Huawei/DevEco Studio"
DEVECO_WSL="/mnt/d/Program Files/Huawei/DevEco Studio"
DEVECO_SDK_HOME="$DEVECO_WIN/sdk"
DEVECO_NODE="$DEVECO_WSL/tools/node/node.exe"
DEVECO_HVIGOR="$DEVECO_WIN/tools/hvigor/bin/hvigorw.js"
NDK_BASE="$DEVECO_WSL/sdk/default/openharmony/native"
NDK_BIN="$NDK_BASE/llvm/bin"
NDK_SYSROOT="$NDK_BASE/sysroot"

export DEVECO_SDK_HOME
export PATH="$NDK_BIN:$PATH"
# hvigor needs a writable Windows home dir (not \\wsl$\...)
export HOME="/mnt/d"

echo "[env:wsl] PROJECT_ROOT=$PROJECT_ROOT"
echo "[env:wsl] HMOS_ROOT=$HMOS_ROOT"
echo "[env:wsl] LIBS_DIR=$LIBS_DIR"
echo "[env:wsl] NDK_BIN=$NDK_BIN"
