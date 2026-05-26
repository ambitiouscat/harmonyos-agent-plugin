@echo off
cd /d "%~dp0\..\agent_core"
wasm-pack build --target web --out-dir ../pkg/wasm -- --features wasm
