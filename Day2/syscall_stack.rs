// SPDX-License-Identifier: GPL-2.0
//! A Rust kernel module to print calling stack.

use kernel::prelude::*;

module! {
    type: SyscallStack,
    name: "syscall_stack",
    author: "Fei",
    description: "Dump kernel stack on insmod/rmmod, then sleep 5s",
    license: "GPL",
}

struct SyscallStack;

impl kernel::Module for SyscallStack {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("syscall_stack: insmod -> dump init_module stack\n");

        // SAFETY: dump_stack() 只读取当前栈并打印，进程上下文安全
        unsafe {
            kernel::bindings::dump_stack();
        }

        Ok(SyscallStack)
    }
}

impl Drop for SyscallStack {
    fn drop(&mut self) {
        pr_info!("syscall_stack: rmmod -> dump delete_module stack\n");

        // SAFETY: 同上
        unsafe {
            kernel::bindings::dump_stack();

            kernel::bindings::msleep(5000);

            kernel::bindings::dump_stack();
        }

        // pr_info!("\n");
        //
        // unsafe {
        //     kernel::bindings::msleep(5000);
        // }
        //
        // pr_info!("\n");
    }
}