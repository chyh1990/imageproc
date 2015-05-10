// build.rs

use std::process::Command;
use std::env;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(e) => panic!("{} return the error {}", stringify!($e), e),
    })
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    assert!(t!(cmd.status()).success());
}

fn make() -> &'static str {
    if cfg!(target_os = "freebsd") {"gmake"} else {"make"}
}

fn main() {
    let proj_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    run(Command::new(make())
        .arg("-j4")
        .current_dir("3rdparty/FreeImage"));
    run(Command::new("cp")
        .arg("3rdparty/FreeImage/Dist/libfreeimage.a")
        .arg(format!("{}/", out_dir)));

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-flags=-l dylib=stdc++");
}
