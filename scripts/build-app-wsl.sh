#!/usr/bin/env bash
# Build full HAP from WSL2 — sources env-setup-wsl.sh.
# Usage:
#   ./scripts/build-app-wsl.sh              # debug
#   ./scripts/build-app-wsl.sh release      # release
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup-wsl.sh"
BUILD_MODE="${1:-debug}"

cd "$HMOS_ROOT"
echo "=== Building full app ($BUILD_MODE) [WSL] ==="
"$DEVECO_NODE" "$DEVECO_HVIGOR" \
    --mode project \
    -p product=default \
    -p buildMode="$BUILD_MODE" \
    assembleApp \
    --no-daemon \
    --stacktrace

HAP_PATH="$HMOS_ROOT/entry/build/default/outputs/default/entry-default-signed.hap"
if [ -f "$HAP_PATH" ]; then
    ls -lh "$HAP_PATH"
else
    echo "ERROR: HAP not found at $HAP_PATH"
    exit 1
fi
echo "=== App build done ==="
