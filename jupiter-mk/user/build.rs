fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg=-T{}/user.ld", manifest);
    println!("cargo:rustc-link-arg=-no-pie");
    println!("cargo:rerun-if-changed=user.ld");
}
