/*
 * Copyright Redis Ltd. 2022 - present
 * Licensed under your choice of the Redis Source Available License 2.0 (RSALv2) or
 * the Server Side Public License v1 (SSPLv1).
 */

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

lazy_static::lazy_static! {
    static ref ARCH: &'static str = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => panic!("Given arch are not support: {}", std::env::consts::ARCH),
    };

    static ref  OS: &'static str = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "apple-darwin",
        _ => panic!("Os '{}' are not supported", std::env::consts::OS),
    };

    static ref PROFILE: String = env::var("PROFILE").expect("PROFILE env var was not given");

    static ref V8_DEFAULT_VERSION: &'static str = "11.8.172.16";
    static ref V8_VERSION: String = env::var("V8_VERSION").map(|v| if v == "default" {V8_DEFAULT_VERSION.to_string()} else {v}).unwrap_or(V8_DEFAULT_VERSION.to_string());
    static ref V8_HEADERS_PATH: String = env::var("V8_HEADERS_PATH").unwrap_or("v8_c_api/libv8.include.zip".into());
    static ref V8_HEADERS_URL: String = env::var("V8_HEADERS_URL").unwrap_or(format!("http://redismodules.s3.amazonaws.com/redisgears/dependencies/libv8.{}.include.zip", *V8_VERSION));
    static ref V8_MONOLITH_PATH: String = env::var("V8_MONOLITH_PATH").unwrap_or(format!("v8_c_api/libv8_monolith_{}.a", *PROFILE));
    static ref V8_MONOLITH_URL: String = env::var("V8_MONOLITH_URL").unwrap_or(format!("http://redismodules.s3.amazonaws.com/redisgears/dependencies/libv8_monolith.{}.{}.{}.{}.a", *V8_VERSION, *ARCH, *PROFILE, *OS));

    static ref V8_HEADERS_DIRECTORY: &'static str = "v8_c_api/src/v8include/";
    static ref LIBV8_PATH: &'static str = "v8_c_api/src/libv8.a";

    static ref V8_UPDATE_HEADERS: bool = env::var("V8_UPDATE_HEADERS").map(|v| v == "yes").unwrap_or(false);
    static ref V8_FORCE_HEADERS_DOWNLOAD: bool = env::var("V8_FORCE_DOWNLOAD_V8_HEADERS").map(|v| v == "yes").unwrap_or(false);
    static ref V8_FORCE_MONOLITH_DOWNLOAD: bool = env::var("V8_FORCE_DOWNLOAD_V8_MONOLITH").map(|v| v == "yes").unwrap_or(false);
}

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

    if *V8_UPDATE_HEADERS {
        // download and update headers
        if *V8_FORCE_HEADERS_DOWNLOAD {
            run_cmd("rm", &["-rf", &V8_HEADERS_PATH]);
        }
        if !Path::new(V8_HEADERS_PATH.as_str()).exists() {
            run_cmd("wget", &["-O", &V8_HEADERS_PATH, &V8_HEADERS_URL]);
        }

        run_cmd("rm", &["-rf", *V8_HEADERS_DIRECTORY]);
        run_cmd("mkdir", &["-p", *V8_HEADERS_DIRECTORY]);
        run_cmd("unzip", &[&V8_HEADERS_PATH, "-d", *V8_HEADERS_DIRECTORY]);
    }

    run_cmd("make", &["-C", "v8_c_api/"]);

    let output_dir = env::var("OUT_DIR").expect("Can not find out directory");

    run_cmd("cp", &[*LIBV8_PATH, &output_dir]);

    if *V8_FORCE_MONOLITH_DOWNLOAD {
        run_cmd("rm", &["-rf", &V8_MONOLITH_PATH]);
    }

    if !Path::new(V8_MONOLITH_PATH.as_str()).exists() {
        // download libv8_monolith.a
        run_cmd("wget", &["-O", &V8_MONOLITH_PATH, &V8_MONOLITH_URL]);
    }

    run_cmd("cp", &[&V8_MONOLITH_PATH, &output_dir]);

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
                "cargo:rustc-flags=-L{} -lv8 -lv8_monolith_{} -ldl -lc",
                output_dir, *PROFILE
            );
            println!("cargo:rustc-cdylib-link-arg=-Wl,-Bstatic");
            println!("cargo:rustc-cdylib-link-arg=-lstdc++");
            println!("cargo:rustc-cdylib-link-arg=-Wl,-Bdynamic");
        }
        "macos" => {
            println!(
                "cargo:rustc-flags=-L{} -lv8 -lv8_monolith_{} -lc++ -ldl -lc",
                output_dir, *PROFILE
            );
        }
        _ => panic!("Os '{}' are not supported", std::env::consts::OS),
    }

    vergen::EmitBuilder::builder()
        .all_git()
        .emit()
        .expect("vergen failed.");

    println!("cargo:rustc-env=PROFILE={}", *PROFILE);
}
