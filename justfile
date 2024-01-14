build-frontend profile='debug':
    @echo "Building frontend..."
    cargo build -p frontend --target=wasm32-unknown-unknown --target-dir=target_frontend
    wasm-bindgen target_frontend/wasm32-unknown-unknown/debug/frontend.wasm --target web --keep-debug --no-demangle --out-dir dudger
    