## cross-compile-wsl

### Purpose
WSL2-based cross-compilation pipeline producing musl-target static libraries (`libagent_core.a`) for both x86_64 (simulator) and aarch64 (device), including support for C-dependent crates (ring via rustls → ureq).

### Requirements

- **REQ-CC-001**: WSL2 Ubuntu SHALL host the Rust toolchain (1.95.0+) with `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` targets
- **REQ-CC-002**: x86_64 ring C code SHALL be compiled with native Linux `gcc` (no cross-compilation issues, ring is pure crypto)
- **REQ-CC-003**: aarch64 ring C code SHALL be compiled with OHOS NDK `clang.exe` via shell wrappers (`~/bin/aarch64-ohos-clang`)
- **REQ-CC-004**: Both targets SHALL link via OHOS NDK clang with `--sysroot`, `-fuse-ld=lld`, `-static`, `+crt-static`
- **REQ-CC-005**: Source SHALL be synced to WSL native filesystem (`~/hmos_rust_build`) via rsync or cp, avoiding `/mnt/d/` cross-filesystem I/O issues
- **REQ-CC-006**: NDK SHALL be accessible via symlink `~/ohos-ndk → /mnt/d/.../native` to avoid space-in-path issues
- **REQ-CC-007**: OHOS clang wrappers SHALL properly quote `$SOURCE/clang.exe` paths to handle spaces
- **REQ-CC-008**: Output `.a` files SHALL be copied to `hmosagent/libs/{x86_64,arm64-v8a}/libagent_core.a`
- **REQ-CC-009**: Build SHALL be reproducible via `scripts/cross-compile.sh [x86_64|aarch64]`

### Implementation

- `.cargo/config.toml`: `CC_x86_64 = "gcc"`, `CC_aarch64 = clang wrapper`, linker = OHOS clang
- `scripts/env-setup.sh`: NDK/DevEco path resolution for Git Bash + WSL
- `scripts/cross-compile.sh`: rsync source → write cargo config → cargo build → copy .a
- `scripts/build-all.sh`: Git Bash orchestrator (WSL cross-compile + Windows HAP build)

### Troubleshooting (见编译疑难杂症笔记 Q79g1zW8j0vc)
- OHOS clang shell wrapper: `exec $SOURCE/clang` → spaces in path → fix with `"$SOURCE/clang.exe"`
- Windows PE clang.exe: WSL binfmt can't write to `/mnt/d/` → build in `~/`
- musl sysroot: only has `x86_64-linux-ohos/bits/alltypes.h`, not `x86_64-linux-musl/` → use gcc for x86_64
