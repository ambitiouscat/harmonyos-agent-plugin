#!/usr/bin/env bash
# Build the hmosagent HAP via hvigor
# Usage:
#   ./scripts/build-hap.sh              # debug build
#   ./scripts/build-hap.sh release      # release build
#   ./scripts/build-hap.sh debug        # debug build (explicit)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup.sh"

BUILD_MODE="${1:-debug}"

cd "$HMOS_ROOT"

echo ""
echo "=== Building hmosagent HAP ($BUILD_MODE) ==="

"$DEVECO_NODE" "$DEVECO_HVIGOR" \
    --mode module \
    -p module=entry@default \
    -p buildMode="$BUILD_MODE" \
    assembleHap \
    --no-daemon

echo ""
echo "=== HAP build done ==="

# Show output HAP location
HAP_DIR="$HMOS_ROOT/entry/build/default/outputs/default"
if [ -f "$HAP_DIR/entry-default-signed.hap" ]; then
    ls -lh "$HAP_DIR/entry-default-signed.hap"
else
    ls -lh "$HAP_DIR/"*.hap 2>/dev/null || echo "(HAP not found in $HAP_DIR)"
fi
