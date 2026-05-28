#!/usr/bin/env bash
# Cross-compile Rust agent_core for OHOS (musl targets) via WSL2.
#
# IMPORTANT: Must run from WSL2, not from Windows bash.
# The build uses gcc (native Linux) for ring's C code to avoid
# OHOS clang.exe cross-filesystem I/O issues.
#
# Usage (from WSL):
#   ./scripts/cross-compile.sh              # both targets
#   ./scripts/cross-compile.sh x86_64       # x86_64 only (simulator)
#   ./scripts/cross-compile.sh aarch64      # aarch64 only (device)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LIBS_DIR="$PROJECT_ROOT/hmosagent/libs"
RUST_SRC="$PROJECT_ROOT/rust_core"

# WSL native build directory (avoids /mnt/d/ cross-fs I/O issues with clang.exe)
WSL_BUILD_DIR="$HOME/hmos_rust_build"

# OHOS NDK (accessible from WSL via symlink ~/ohos-ndk)
OHOS_NDK="$HOME/ohos-ndk"
OHOS_BIN="$OHOS_NDK/llvm/bin"
OHOS_SYSROOT="$OHOS_NDK/sysroot"

# Ensure NDK is accessible
if [ ! -f "$OHOS_BIN/clang.exe" ]; then
    echo "ERROR: OHOS NDK not found at $OHOS_NDK"
    echo "  Create symlink first: ln -s '/mnt/d/Program Files/Huawei/DevEco Studio/sdk/default/openharmony/native' ~/ohos-ndk"
    exit 1
fi

# Create clang wrappers if needed
mkdir -p "$HOME/bin"

cat > "$HOME/bin/aarch64-ohos-clang" << 'WRAPEOF'
#!/bin/sh
exec "$HOME/ohos-ndk/llvm/bin/clang.exe" --target=aarch64-unknown-linux-ohos "$@"
WRAPEOF
chmod +x "$HOME/bin/aarch64-ohos-clang"

cat > "$HOME/bin/x86_64-ohos-clang" << 'WRAPEOF'
#!/bin/sh
exec "$HOME/ohos-ndk/llvm/bin/clang.exe" --target=x86_64-unknown-linux-ohos "$@"
WRAPEOF
chmod +x "$HOME/bin/x86_64-ohos-clang"

cat > "$HOME/bin/ohos-ar" << 'WRAPEOF'
#!/bin/sh
exec "$HOME/ohos-ndk/llvm/bin/llvm-ar.exe" "$@"
WRAPEOF
chmod +x "$HOME/bin/ohos-ar"

# Copy source to WSL native filesystem (avoid /mnt/d/ cross-fs issues)
echo "=== Syncing source to WSL native ==="
rsync -a --delete "$RUST_SRC/" "$WSL_BUILD_DIR/" --exclude target 2>/dev/null || {
    rm -rf "$WSL_BUILD_DIR"
    cp -r "$RUST_SRC" "$WSL_BUILD_DIR"
    rm -rf "$WSL_BUILD_DIR/target"
}

# Write cargo config for WSL cross-compilation
# Key: gcc for x86_64 ring C code (pure crypto, no libc needed)
#      OHOS clang for aarch64 and for linking both targets
mkdir -p "$WSL_BUILD_DIR/.cargo"
cat > "$WSL_BUILD_DIR/.cargo/config.toml" << CFGEOF
# WSL cross-compilation config
# x86_64: gcc (native) for ring C code; OHOS clang for linking
# aarch64: native gcc for ring C code; OHOS clang for linking

[env]
CC_aarch64_unknown_linux_musl = "aarch64-linux-gnu-gcc"
AR_aarch64_unknown_linux_musl = "aarch64-linux-gnu-ar"
CC_x86_64_unknown_linux_musl = "gcc"
AR_x86_64_unknown_linux_musl = "ar"

[target.aarch64-unknown-linux-musl]
linker = "$HOME/bin/aarch64-ohos-clang"
rustflags = [
    "-C", "link-arg=--sysroot=$OHOS_SYSROOT",
    "-C", "link-arg=-fuse-ld=lld",
    "-C", "link-arg=-static",
    "-C", "target-feature=+crt-static",
]

[target.x86_64-unknown-linux-musl]
linker = "$HOME/bin/x86_64-ohos-clang"
rustflags = [
    "-C", "link-arg=--sysroot=$OHOS_SYSROOT",
    "-C", "link-arg=-fuse-ld=lld",
    "-C", "link-arg=-static",
    "-C", "target-feature=+crt-static",
]
CFGEOF

cd "$WSL_BUILD_DIR"

# Determine targets
TARGETS=()
if [ $# -eq 0 ] || [ "$1" = "all" ]; then
    TARGETS=("x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl")
elif [ "$1" = "x86_64" ]; then
    TARGETS=("x86_64-unknown-linux-musl")
elif [ "$1" = "aarch64" ]; then
    TARGETS=("aarch64-unknown-linux-musl")
else
    echo "Usage: $0 [all|x86_64|aarch64]"
    exit 1
fi

BUILD_FAILED=0
for TARGET in "${TARGETS[@]}"; do
    echo ""
    echo "=== Cross-compiling for $TARGET ==="

    if ! cargo build --release --target "$TARGET"; then
        echo "!! FAILED: $TARGET"
        BUILD_FAILED=1
        continue
    fi

    STATIC_LIB="$WSL_BUILD_DIR/target/$TARGET/release/libagent_core.a"
    if [ ! -f "$STATIC_LIB" ]; then
        echo "!! FAILED: $STATIC_LIB not found"
        BUILD_FAILED=1
        continue
    fi

    # Determine output directory on Windows side
    if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
        DEST_ABI="x86_64"
    else
        DEST_ABI="arm64-v8a"
    fi
    DEST_DIR="$LIBS_DIR/$DEST_ABI"

    mkdir -p "$DEST_DIR"
    cp -v "$STATIC_LIB" "$DEST_DIR/libagent_core.a"
    ls -lh "$DEST_DIR/libagent_core.a"
    echo "  -> $TARGET build OK"
done

echo ""
echo "============================================"
if [ "$BUILD_FAILED" -eq 0 ]; then
    echo "  All targets built successfully!"
else
    echo "  WARNING: Some targets failed to build."
fi
echo ""
echo "  Output files:"
for TARGET in "${TARGETS[@]}"; do
    if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
        DEST_ABI="x86_64"
    else
        DEST_ABI="arm64-v8a"
    fi
    DEST_FILE="$LIBS_DIR/$DEST_ABI/libagent_core.a"
    if [ -f "$DEST_FILE" ]; then
        SIZE=$(ls -lh "$DEST_FILE" | awk '{print $5}')
        echo "    $DEST_ABI  $SIZE  $DEST_FILE"
    else
        echo "    $DEST_ABI  MISSING"
    fi
done
echo "============================================"
