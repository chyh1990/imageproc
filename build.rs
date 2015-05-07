// build.rs

// use std::process::Command;
use std::env;

fn main() {
    let proj_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rustc-link-search=native={}/3rdparty", proj_dir);
}
