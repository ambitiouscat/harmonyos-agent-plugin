#!/usr/bin/env bash
# Build full HAP (HAR + entry) — no cross-compile (uses pre-built .a files).
# Usage:
#   ./scripts/build-app.sh              # debug
#   ./scripts/build-app.sh release      # release
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup.sh"
BUILD_MODE="${1:-debug}"

cd "$HMOS_ROOT"
echo "=== Building full app ($BUILD_MODE) ==="
"$DEVECO_NODE" "$DEVECO_HVIGOR" \
    --mode project \
    -p product=default \
    -p buildMode="$BUILD_MODE" \
    assembleApp \
    --no-daemon

HAP_PATH="$HMOS_ROOT/entry/build/default/outputs/default/entry-default-signed.hap"
if [ -f "$HAP_PATH" ]; then
    ls -lh "$HAP_PATH"
else
    echo "ERROR: HAP not found at $HAP_PATH"
    exit 1
fi
echo "=== App build done ==="
