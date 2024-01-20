#![feature(absolute_path)]

use std::{env, path::PathBuf};

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let backend_manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let fe_target_dir = backend_manifest_dir.join("../target_frontend/");
    let fe_platform_dir = fe_target_dir.join("wasm32-unknown-unknown");
    let fe_profile_dir = fe_platform_dir.join(profile);
    let fe_bindgen_dir = fe_profile_dir.join("bindgen_out");

    println!(
        "cargo:rustc-env=HAPPENINGS_WASM={}",
        fe_bindgen_dir.join("frontend_bg.wasm").to_string_lossy()
    );

    println!(
        "cargo:rustc-env=HAPPENINGS_JS={}",
        fe_bindgen_dir.join("frontend.js").to_string_lossy()
    );
}

