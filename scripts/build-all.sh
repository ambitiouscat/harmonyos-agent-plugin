#!/usr/bin/env bash
# Full build: cross-compile Rust (WSL) + build HAP (Git Bash)
# MUST be run from Git Bash on Windows (not WSL directly).
# Usage: ./scripts/build-all.sh [x86_64|aarch64] [debug|release]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

TARGET="${1:-x86_64}"
BUILD_MODE="${2:-debug}"

echo "=== Full build: target=$TARGET mode=$BUILD_MODE ==="

# Step 1: Cross-compile Rust in WSL
echo "=== Step 1/2: Cross-compile Rust (WSL) ==="
wsl -e bash -c "
export PATH=\"\$HOME/.cargo/bin:\$PATH\"
cd /mnt/d/work/ai_work_space/vibecoding2harmoney/ai-agent-harmoney
bash scripts/cross-compile.sh $TARGET
"

# Step 2: Build HAP via hvigor (Windows)
echo "=== Step 2/2: Build HAP ==="
bash "$SCRIPT_DIR/build-hap.sh" "$BUILD_MODE"

echo ""
echo "=== Full build complete ==="
