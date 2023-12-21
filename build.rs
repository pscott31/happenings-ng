use std::{env, path::PathBuf};
use wasm_bindgen_cli_support::Bindgen;

// use anyhow::{ensure, Context};
// use std::process::{Command, Stdio};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").map(PathBuf::from)?;
    let debug = env::var("PROFILE")? == "debug";

    // let cargo = env::var("CARGO")?;
    // let fe_dir = out_dir.join("frontend");
    //     println!("cargo:rerun-if-changed=frontend");
    //     let output = cmd
    //         .arg("build")
    //         .arg("--manifest-path=frontend/Cargo.toml")
    //         .arg("--target=wasm32-unknown-unknown")
    //         .arg("--target-dir")
    //         .arg(fe_dir.as_os_str())
    //         .stderr(Stdio::inherit())
    //         .output()
    //         .context("failed call `cargo` build for wasm")?;

    //     ensure!(output.status.success(), "`cargo` invocation failed");

    // for (key, value) in env::vars() {
    //     println!("cargo:warning {key}: {value}");
    // }

    // let wasm_in = fe_dir.join("wasm32-unknown-unknown/debug/frontend.wasm");
    let wasm_in = env::var("CARGO_CDYLIB_FILE_FRONTEND").map(PathBuf::from)?;
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

