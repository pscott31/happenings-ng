use std::{env, path::PathBuf};

fn main() {
    let profile = env::var("PROFILE")?;
    let fe_target_dir: PathBuf = "../../target_frontend/wasm32-unknown-unknown".into();

    let foo = out_dir.join(profile);
    let bar = foo.join(bindgen_out);
    let baz = bar.join("frontend_bg.wasm");
    println!("cargo:warning={baz}");

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

