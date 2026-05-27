#!/usr/bin/env bash
# Build Node.js native addon from Rust core.
# Usage: ./scripts/build-node.sh
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup.sh"

cd "$RUST_ROOT/agent_core"
echo "=== Building Node addon (release, features=node) ==="
cargo build --release --features node

echo "=== Node build done ==="
LIB_PATH="$RUST_ROOT/target/release/agent_core.dll"
if [ -f "$LIB_PATH" ]; then
    ls -lh "$LIB_PATH"
fi
