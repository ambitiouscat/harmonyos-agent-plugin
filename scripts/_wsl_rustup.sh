#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"
echo "Current version:"
rustc --version
echo "Updating..."
rustup update stable 2>&1
echo "New version:"
rustc --version
echo "DONE"
