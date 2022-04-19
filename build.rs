extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.h");
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.cpp");

    let res = Command::new("make").args(&["-C", "v8_c_api/"]).output().expect("failed to compile v8_c_api");
    println!("{}", String::from_utf8(res.stdout).unwrap());

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

    println!("cargo:rustc-flags=-L./v8_c_api/src/ -L./v8_c_api/ -lv8 -lv8_monolith -lstdc++")
}
