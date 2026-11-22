//! User programs embedded into the kernel image.
//!
//! The Makefile builds `../user` before the kernel, so these paths always
//! exist when the kernel is compiled. A future milestone replaces this table
//! with a proper initrd / boot-module loader.

pub const CONSOLE: usize = 0;
pub const SHELL: usize = 1;
pub const HELLO: usize = 2;

macro_rules! user_bin {
    ($name:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../user/target/riscv64gc-unknown-none-elf/release/",
            $name
        ))
    };
}

pub fn image(id: usize) -> Option<&'static [u8]> {
    match id {
        CONSOLE => Some(user_bin!("console")),
        SHELL => Some(user_bin!("shell")),
        HELLO => Some(user_bin!("hello")),
        _ => None,
    }
}

pub fn name(id: usize) -> &'static str {
    match id {
        CONSOLE => "console",
        SHELL => "shell",
        HELLO => "hello",
        _ => "unknown",
    }
}
