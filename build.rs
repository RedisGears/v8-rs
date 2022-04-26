extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.h");
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.cpp");

    if !Command::new("make")
        .args(&["-C", "v8_c_api/"])
        .status()
        .expect("failed to compile v8_c_api")
        .success()
    {
        panic!("failed to compile v8_c_api");
    }

    let output_dir = env::var("OUT_DIR").expect("Can not find out directory");

    if !Command::new("cp")
        .args(&["v8_c_api/src/libv8.a", &output_dir])
        .status()
        .expect("failed copy libv8.a to output directory")
        .success()
    {
        panic!("failed copy libv8.a to output directory");
    }

    let v8_monolith_path = match env::var("V8_MONOLITH_PATH") {
        Ok(path) => path,
        Err(_) => "v8_c_api/libv8_monolith.a".to_string(),
    };

    if !Command::new("cp")
        .args(&[&v8_monolith_path, &output_dir])
        .status()
        .expect("failed copy libv8_monolith.a to output directory")
        .success()
    {
        panic!("failed copy libv8_monolith.a to output directory");
    }

    let build = bindgen::Builder::default();

    let bindings = build
        .header("v8_c_api/src/v8_c_api.h")
        .size_t_is_usize(true)
        .layout_tests(false)
        .generate()
        .expect("error generating bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("v8_c_bindings.rs"))
        .expect("failed to write bindings to file");

    println!(
        "cargo:rustc-flags=-L{} -lv8 -lv8_monolith -lstdc++",
        output_dir
    );
}
