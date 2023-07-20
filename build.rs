// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::env;
use std::process::Command;

fn try_to_find_and_link_lib(lib_name: &str) -> bool {
    println!("cargo:rerun-if-env-changed={lib_name}_COMPILE");
    if let Ok(v) = env::var(format!("{lib_name}_COMPILE")) {
        if v.to_lowercase() == "true" || v == "1" {
            return false;
        }
    }

    println!("cargo:rerun-if-env-changed={lib_name}_LIB_DIR");
    println!("cargo:rerun-if-env-changed={lib_name}_STATIC");

    if let Ok(lib_dir) = env::var(format!("{lib_name}_LIB_DIR")) {
        println!("cargo:rustc-link-search=native={lib_dir}");
        let mode = match env::var_os(format!("{lib_name}_STATIC")) {
            Some(_) => "static",
            None => "dylib",
        };
        println!("cargo:rustc-link-lib={}={}", mode, lib_name.to_lowercase());
        return true;
    }
    false
}

fn build_snappy() {
    let target = env::var("TARGET").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    Command::new("cmake")
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-S snappy")
        .arg(format!("-B {out_dir}"))
        .output()
        .expect("Failed to execute CMake command");

    let endianness = env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
    let mut config = cc::Build::new();

    config.include("snappy/");
    config.include(".");
    config.include(out_dir);
    config.define("NDEBUG", Some("1"));
    config.extra_warnings(false);

    if target.contains("msvc") {
        config.flag("-EHsc");
    } else {
        // Snappy requires C++11.
        // See: https://github.com/google/snappy/blob/master/CMakeLists.txt#L32-L38
        config.flag("-std=c++11");
    }

    if endianness == "big" {
        config.define("SNAPPY_IS_BIG_ENDIAN", Some("1"));
    }

    config.file("snappy/snappy.cc");
    config.file("snappy/snappy-sinksource.cc");
    config.file("snappy/snappy-c.cc");
    config.cpp(true);
    config.compile("libsnappy.a");
}

fn main() {
    if !try_to_find_and_link_lib("SNAPPY") {
        println!("cargo:rerun-if-changed=snappy/");
        build_snappy();
    } else {
        let target = env::var("TARGET").unwrap();
        if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
            println!("cargo:rustc-link-lib=dylib=c++");
        } else if target.contains("linux") {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    }
}
