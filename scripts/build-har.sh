#!/usr/bin/env bash
# Build only the hmos_agent_core HAR module (no cross-compile, no HAP).
# Usage:
#   ./scripts/build-har.sh              # debug
#   ./scripts/build-har.sh release      # release
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup.sh"
BUILD_MODE="${1:-debug}"

cd "$HMOS_ROOT"
echo "=== Building HAR ($BUILD_MODE) ==="
"$DEVECO_NODE" "$DEVECO_HVIGOR" \
    --mode module \
    -p module=hmos_agent_core@default \
    -p buildMode="$BUILD_MODE" \
    assembleHar \
    --no-daemon

HAR_PATH="$HMOS_ROOT/hmos_agent_core/build/default/outputs/default/hmos_agent_core.har"
if [ -f "$HAR_PATH" ]; then
    ls -lh "$HAR_PATH"
else
    echo "ERROR: HAR not found at $HAR_PATH"
    exit 1
fi
echo "=== HAR build done ==="
