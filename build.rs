extern crate serde_codegen;

use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    if !Path::new("rocksdb/.git").exists() {
        assert!(Command::new("git").args(&["submodule", "update", "--init"]).status().unwrap().success());
    }
    assert!(Command::new("make").arg("shared_lib").env("PORTABLE", "1").current_dir("rocksdb").status().unwrap().success());
    println!("cargo:rustc-link-search=rocksdb");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let src = Path::new("src/serde_types.in.rs");
    let dst = Path::new(&out_dir).join("serde_types.rs");
    serde_codegen::expand(&src, &dst).unwrap();
}
