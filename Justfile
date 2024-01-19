
alias bfe := build-frontend
alias bbe := build-backend
alias b := build
alias r := run

profile := "debug"
# bindgen_flags := if profile=="debug" {"--keep-debug --no-demangle"} else {""}
bindgen_flags := if profile=="debug" {""} else {""} 
cargo_flags := if profile=="release" {"--release"} else {""}

frontend_target_dir := "target_frontend"
wasm_out_dir := frontend_target_dir + "/wasm32-unknown-unknown/" + profile 
bindgen_out_dir := wasm_out_dir + "/bindgen_out"

export HAPPENINGS_WASM := absolute_path(bindgen_out_dir + "/frontend_bg.wasm")
export HAPPENINGS_JS := absolute_path(bindgen_out_dir + "/frontend.js")

build-frontend:
    cargo build -p frontend {{cargo_flags}} --target=wasm32-unknown-unknown --target-dir {{frontend_target_dir}}
    wasm-bindgen {{wasm_out_dir}}/frontend.wasm --web {{bindgen_flags}} --out-dir {{bindgen_out_dir}} 

build-backend:
    echo ${HAPPENINGS_JS}
    echo ${HAPPENINGS_WASM}
    cargo build {{cargo_flags}}

build: build-frontend build-backend

run: build
    ./target/{{profile}}/happenings

db_client:
    surreal sql --db happenings --ns happenings

db:
    surreal start --user root --pass root file:testdata.db

cargo *args:
    cargo {{args}}

