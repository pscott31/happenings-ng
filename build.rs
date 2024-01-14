use std::{env, path::PathBuf};
use wasm_bindgen_cli_support::Bindgen;

use anyhow::{ensure, Context};
use std::process::{Command, Stdio};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").map(PathBuf::from)?;
    let profile = env::var("PROFILE")?;
    let debug = profile == "debug";

    let cargo = env::var("CARGO")?;

    // for (key, value) in env::vars() {
    //     println!("cargo:warning={key}:{value}");
    // }

    // This is a bit horrible; cargo sets a bunch of environment variables that
    // we don't want to pass to the `cargo` invocation below; in parcicular the
    // CARGO_ENCODED_RUSTFLAGS causes us to try and use 'ldd' even when it's configured
    // in cargo config only for [target.x86_64-unknown-linux-gnu] when linking WASM.
    // Once build artificats for different targets work properly this build script will not be needed

    let filtered_env: std::collections::HashMap<String, String> = env::vars()
        .filter(|&(ref k, _)| !k.starts_with("CARGO"))
        .collect();

    // for (key, value) in filtered_env.iter() {
    //     println!("cargo:warning=AFTER_{key}:{value}");
    // }

    let fe_dir = out_dir.join("frontend");
    println!("cargo:rerun-if-changed=frontend");

    let release_args = if debug { vec![] } else { vec!["--release"] };

    let output = Command::new(cargo)
        .arg("build")
        .args(release_args)
        .arg("--manifest-path=frontend/Cargo.toml")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--target-dir")
        .arg(fe_dir.as_os_str())
        .stderr(Stdio::inherit())
        .env_clear()
        .envs(&filtered_env)
        .output()
        .context("failed call `cargo` build for wasm")?;

    ensure!(output.status.success(), "`cargo` invocation failed");

    let wasm_in = fe_dir.join(format!("wasm32-unknown-unknown/{}/frontend.wasm", profile));
    // let wasm_in = env::var("CARGO_CDYLIB_FILE_FRONTEND").map(PathBuf::from)?;
    let bindgen_out = out_dir.join("frontend_bg");

    println!("cargo:warning=WASM INPUT: {wasm_in:?}");
    println!("cargo:warning=BINDGEN OUT: {bindgen_out:?}");
    Bindgen::new()
        .input_path(wasm_in)
        .web(true)
        .unwrap()
        .keep_debug(debug)
        .demangle(!debug)
        .generate(bindgen_out.as_os_str())
        .unwrap();

    println!(
        "cargo:rustc-env=HAPPENINGS_WASM={}",
        bindgen_out.join("frontend_bg.wasm").to_string_lossy()
    );

    println!(
        "cargo:rustc-env=HAPPENINGS_JS={}",
        bindgen_out.join("frontend.js").to_string_lossy()
    );
    Ok(())
}

