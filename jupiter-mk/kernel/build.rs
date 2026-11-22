use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let board = if env::var_os("CARGO_FEATURE_BOARD_JUPITER").is_some() {
        "jupiter"
    } else {
        "qemu"
    };

    let linker = manifest.join(format!("linker-{board}.ld"));
    println!("cargo:rustc-link-arg=-T{}", linker.display());
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_BOARD_JUPITER");
    println!("cargo:rerun-if-changed={}", linker.display());
}
