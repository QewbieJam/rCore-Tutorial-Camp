// SPDX-License-Identifier: GPL-2.0
//! Day1: hello -> sleep 5s -> goodbye.

use kernel::prelude::*;

module! {
    type: RustHelloWorld,
    name: "rust_hello",
    author: "Fei",
    description: "Simple module",
    license: "GPL",
}

struct RustHelloWorld;

impl kernel::Module for RustHelloWorld {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello World from Rust kernel module!\n");
        pr_info!("Now start to sleep!\n");

        // SAFETY: init 运行在进程上下文，允许睡眠。
        unsafe { kernel::bindings::msleep(5000) };

        Ok(RustHelloWorld)
    }
}

impl Drop for RustHelloWorld {
    fn drop(&mut self) {
        pr_info!("Goodbye from Rust kernel module!\n");
    }
}
