use anyhow::{ensure, Context};
use std::{env, path::PathBuf, process::{Command, Stdio}};
use wasm_bindgen_cli_support::Bindgen;

use jobserver::Client;

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").map(PathBuf::from)?;
    let cargo = env::var("CARGO")?;
    let fe_dir = out_dir.join("frontend");
    let debug = env::var("PROFILE")? == "debug";

    println!("cargo:rerun-if-changed=frontend");

    // // See API documentation for why this is `unsafe`
    // let jobserver =
    //     match unsafe { Client::from_env() } {
    //         Some(client) => client,
    //         None => panic!("client not configured"),
    //     };

    let jobserver = Client::new(8).expect("failed to create jobserver");
    let mut cmd = Command::new(cargo);
    jobserver.configure(&mut cmd);

    let output = cmd
        .arg("build")
        .arg("--manifest-path=frontend/Cargo.toml")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--target-dir")
        .arg(fe_dir.as_os_str())
        .stderr(Stdio::inherit())
        .output()
        .context("failed call `cargo` build for wasm")?;

    ensure!(output.status.success(), "`cargo` invocation failed");

    let wasm_in = fe_dir.join("wasm32-unknown-unknown/debug/frontend.wasm");
    let bindgen_out = out_dir.join("frontend_bg");
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

