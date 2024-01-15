

profile := "debug"
# bindgen_flags := if profile=="debug" {"--keep-debug --no-demangle"} else {""}
bindgen_flags := if profile=="debug" {""} else {""}

cargo_flags := if profile=="release" {"--release"} else {""}

build-frontend:
    # release_flag := if profile == "release" {"--release"} else {""}
    cargo build -p frontend {{cargo_flags}} --target=wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/{{profile}}/frontend.wasm {{bindgen_flags}} --out-dir dudger
    