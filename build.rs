/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

extern crate bindgen;

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.h");
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.cpp");

    if !Command::new("make")
        .args(["-C", "v8_c_api/"])
        .status()
        .expect("failed to compile v8_c_api")
        .success()
    {
        panic!("failed to compile v8_c_api");
    }

    let output_dir = env::var("OUT_DIR").expect("Can not find out directory");

    if !Command::new("cp")
        .args(["v8_c_api/src/libv8.a", &output_dir])
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

    let version = "10.8.168.21";

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => panic!("Given arch are not support: {}", std::env::consts::ARCH),
    };

    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "apple-darwin",
        _ => panic!("Os '{}' are not supported", std::env::consts::OS),
    };

    let v8_monolith_url = match env::var("V8_MONOLITH_URL") {
        Ok(path) => path,
        Err(_) => format!("http://redismodules.s3.amazonaws.com/redisgears/dependencies/libv8_monolith.{}.{}.{}.a", version, arch, os),
    };

    if !Path::new(&v8_monolith_path).exists() {
        // download libv8_monolith.a
        if !Command::new("wget")
            .args(["-O", &v8_monolith_path, &v8_monolith_url])
            .status()
            .expect("failed downloading libv8_monolith.a")
            .success()
        {
            panic!("failed downloading libv8_monolith.a");
        }
    }

    if !Command::new("cp")
        .args([&v8_monolith_path, &output_dir])
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

    let out_path = PathBuf::from(&output_dir);
    bindings
        .write_to_file(out_path.join("v8_c_bindings.rs"))
        .expect("failed to write bindings to file");

    match std::env::consts::OS {
        "linux" => {
            /* On linux we will statically link to libstdc++ to be able to run on systems that do not have libstdc++ installed. */
            println!(
                "cargo:rustc-flags=-L{} -lv8 -lv8_monolith -ldl -lc",
                output_dir
            );
            println!("cargo:rustc-cdylib-link-arg=-Wl,-Bstatic");
            println!("cargo:rustc-cdylib-link-arg=-lstdc++");
            println!("cargo:rustc-cdylib-link-arg=-Wl,-Bdynamic");
        }
        "macos" => {
            println!(
                "cargo:rustc-flags=-L{} -lv8 -lv8_monolith -lc++ -ldl -lc",
                output_dir
            );
        }
        _ => panic!("Os '{}' are not supported", std::env::consts::OS),
    }
}
