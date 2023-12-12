use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};
use wasm_bindgen_cli_support::Bindgen;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fe_dir = out_dir.join("frontend");

    println!("Compiling WASM");
    for (key, value) in env::vars() {
        println!("{key}: {value}");
    }
    println!("cargo:warning={:?}", fe_dir);

    //     let output = Command::new("cargo")
    //         .arg("build")
    //         .arg("--package")
    //         .arg("frontend")
    //         .arg("--target")
    //         .arg("wasm32-unknown-unknown")
    //         .arg("--target-dir")
    //         .arg(fe_dir.as_os_str())
    //         .output()
    //         .expect("failed to start wasm build");

    //     println!("status: {}", output.status);
    //     io::stdout().write_all(&output.stdout).unwrap();
    //     io::stderr().write_all(&output.stderr).unwrap();

    //     let wasm_in = fe_dir.join("wasm32-unknown-unknown/debug/frontend.wasm");
    //     let bindgen_out = out_dir.join("frontend_bg");
    //     Bindgen::new()
    //         .input_path(wasm_in)
    //         .web(true)
    //         .unwrap()
    //         .generate(bindgen_out.as_os_str())
    //         .unwrap();

    //     println!(
    //         "cargo:rustc-env=HAPPENINGS_WASM={}",
    //         bindgen_out.join("frontend_bg.wasm").to_string_lossy()
    //     );

    //     println!(
    //         "cargo:rustc-env=HAPPENINGS_JS={}",
    //         bindgen_out.join("frontend.js").to_string_lossy()
    //     );
}
