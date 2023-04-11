/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const V8_VERSION: &str = "10.8.168.21";

fn run_cmd(cmd: &str, args: &[&str]) {
    let failure_message = format!("Failed running command: {} {}", cmd, args.join(" "));
    if !Command::new(cmd)
        .args(args)
        .status()
        .expect(&failure_message)
        .success()
    {
        panic!("{}", failure_message);
    }
}

fn main() {
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.h");
    println!("cargo:rerun-if-changed=v8_c_api/src/v8_c_api.cpp");

    let version = env::var("V8_VERSION").unwrap_or(V8_VERSION.into());

    if let Ok(v8_update_header) = env::var("V8_UPDATE_HEADERS") {
        if v8_update_header == "yes" {
            // download and update headers
            let v8_headers_path =
                env::var("V8_HEADERS_PATH").unwrap_or("v8_c_api/libv8.include.zip".into());
            let force_download_v8_headers = env::var("V8_FORCE_DOWNLOAD_V8_HEADERS")
                .map(|v| v == "yes")
                .unwrap_or(false);
            if force_download_v8_headers {
                run_cmd("rm", &["-rf", &v8_headers_path]);
            }
            if !Path::new(&v8_headers_path).exists() {
                let v8_headers_url = env::var("V8_HEADERS_URL").unwrap_or(format!(
                    "http://redismodules.s3.amazonaws.com/redisgears/dependencies/libv8.{}.include.zip", version));
                run_cmd("wget", &["-O", &v8_headers_path, &v8_headers_url]);
            }

            run_cmd("rm", &["-rf", "v8_c_api/src/v8include/"]);
            run_cmd(
                "unzip",
                &[&v8_headers_path, "-d", "v8_c_api/src/v8include/"],
            );
        }
    }

    run_cmd("make", &["-C", "v8_c_api/"]);

    let output_dir = env::var("OUT_DIR").expect("Can not find out directory");

    run_cmd("cp", &["v8_c_api/src/libv8.a", &output_dir]);

    let force_download_v8_monolith = env::var("V8_FORCE_DOWNLOAD_V8_MONOLITH")
        .map(|v| v == "yes")
        .unwrap_or(false);

    let v8_monolith_path =
        env::var("V8_MONOLITH_PATH").unwrap_or("v8_c_api/libv8_monolith.a".into());

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

    let v8_monolith_url = env::var("V8_MONOLITH_URL").unwrap_or(format!(
        "http://redismodules.s3.amazonaws.com/redisgears/dependencies/libv8_monolith.{}.{}.{}.a",
        version, arch, os
    ));

    if force_download_v8_monolith {
        run_cmd("rm", &["-rf", &v8_monolith_path]);
    }

    if !Path::new(&v8_monolith_path).exists() {
        // download libv8_monolith.a
        run_cmd("wget", &["-O", &v8_monolith_path, &v8_monolith_url]);
    }

    run_cmd("cp", &[&v8_monolith_path, &output_dir]);

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

    vergen::EmitBuilder::builder()
        .all_git()
        .emit()
        .expect("vergen failed.");
}
