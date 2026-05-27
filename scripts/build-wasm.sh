#!/usr/bin/env bash
# Build WASM target via WSL (needs clang for ring crate).
# Usage: ./scripts/build-wasm.sh
set -euo pipefail

echo "=== Building WASM (WSL) ==="
wsl -e bash -c "
export PATH=\"\$HOME/.cargo/bin:\$PATH\"
rustup target add wasm32-unknown-unknown 2>/dev/null || true
cd /mnt/d/work/ai_work_space/vibecoding2harmoney/ai-agent-harmoney/rust_core/agent_core
cargo build --target wasm32-unknown-unknown --features wasm
"

echo "=== WASM build done ==="
WASM_PATH="D:/work/ai_work_space/vibecoding2harmoney/ai-agent-harmoney/rust_core/target/wasm32-unknown-unknown/debug/agent_core.wasm"
if [ -f "$WASM_PATH" ]; then
    ls -lh "$WASM_PATH"
fi
