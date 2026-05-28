#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /root/hmos_rust_build
cargo build --release --target aarch64-unknown-linux-musl 2>&1
echo "EXIT_CODE=$?"
