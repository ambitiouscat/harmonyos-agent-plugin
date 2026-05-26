@echo off
cd /d "%~dp0\..\agent_core"
napi build --release --out-dir ../pkg/node --features node
