#!/usr/bin/env bash
# Build HAR and sync to i3d544 project
# Usage: ./scripts/sync-har.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env-setup.sh"

I3D544_LIBS="D:/work/ai_work_space/vibecoding2harmoney/i3d544-harmony/hap_editor/libs"

echo "=== Building HAR ==="
cd "$HMOS_ROOT"

"$DEVECO_NODE" "$DEVECO_HVIGOR" \
    --mode module \
    -p module=hmos_agent_core@default \
    -p buildMode=debug \
    assembleHar \
    --no-daemon

HAR_SRC="$HMOS_ROOT/hmos_agent_core/build/default/outputs/default/hmos_agent_core.har"

if [ ! -f "$HAR_SRC" ]; then
    echo "ERROR: HAR not found at $HAR_SRC"
    exit 1
fi

echo "=== Syncing HAR to i3d544 ==="
mkdir -p "$I3D544_LIBS"
cp -v "$HAR_SRC" "$I3D544_LIBS/hmos_agent_core.har"
ls -lh "$I3D544_LIBS/hmos_agent_core.har"

echo "=== HAR sync done ==="
