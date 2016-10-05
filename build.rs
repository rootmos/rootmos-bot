use std::process::Command;
use std::path::Path;

fn main() {
    if !Path::new("rocksdb/.git").exists() {
        assert!(Command::new("git").args(&["submodule", "update", "--init"]).status().unwrap().success());
    }
    assert!(Command::new("make").arg("shared_lib").env("PORTABLE", "1").current_dir("rocksdb").status().unwrap().success());
    println!("cargo:rustc-link-search=rocksdb");
}
