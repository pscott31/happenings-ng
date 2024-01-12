#!/bin/bash
echo Building frontend
cargo build --manifest-path=frontend/Cargo.toml --target=wasm32-unknown-unknown --target-dir=target_frontend
echo Bindgen
wasm-bindgen target_frontend/wasm32-unknown-unknown/debug/frontend.wasm --target web --keep-debug --no-demangle --out-dir dudger
echo Building backend
HAPPENINGS_WASM=../dudger/frontend_bg.wasm HAPPENINGS_JS=../dudger/frontend.js cargo build
