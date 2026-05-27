#!/usr/bin/env bash
# Run Rust unit + integration tests.
# Usage: ./scripts/test-rust.sh
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup.sh"

cd "$RUST_ROOT/agent_core"
echo "=== Running Rust tests ==="
cargo test
echo "=== Tests done ==="
