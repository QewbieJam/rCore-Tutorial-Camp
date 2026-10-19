# Day2：Rust 模块打印系统调用栈

## 目标

在 Day1 的模块基础上，用 `dump_stack()` 打印**系统调用栈**：

* `insmod` 时打印 `finit_module` 系统调用的内核栈；
* `rmmod` 时打印 `delete_module` 系统调用的内核栈。

## 原理

`insmod` / `rmmod` 本身就是用户态程序，它们分别调用：

| 动作 | 系统调用 | x86_64 编号 |
|---|---|---|
| `insmod` | `finit_module` | 313（0x139） |
| `rmmod` | `delete_module` | 176（0xb0） |

模块的 `init` / `Drop` 就运行在这两个系统调用的内核上下文中，所以在里面调用 `dump_stack()`，打出来的就是该系统调用的完整内核栈。

## 代码

`syscall_stack.rs`：

```rust
//! Day2: dump syscall stack on module load/unload.
use kernel::prelude::*;

module! {
    type: SyscallStack,
    name: "syscall_stack",
    author: "Fei",
    description: "Dump kernel stack on insmod/rmmod",
    license: "GPL",
}

struct SyscallStack;

impl kernel::Module for SyscallStack {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("syscall_stack: insmod -> dump init_module stack\n");
        unsafe { kernel::bindings::dump_stack() };
        Ok(SyscallStack)
    }
}

impl Drop for SyscallStack {
    fn drop(&mut self) {
        pr_info!("syscall_stack: rmmod -> dump delete_module stack\n");
        unsafe { kernel::bindings::dump_stack() };
    }
}
```

## 编译与运行

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.83.0

make LLVM=1 KDIR=$HOME/linux-6.12.107
./scripts/mkinitramfs.sh
./scripts/run_qemu.sh
```

## 输出解读

`insmod` 的栈（从下往上）：

```
entry_SYSCALL_64_after_hwframe   ← 用户态进内核的入口
do_syscall_64                    ← 系统调用分发
__se_sys_finit_module            ← 313 号系统调用
do_init_module                   ← 模块初始化
do_one_initcall
init_module+0x57/0xff0 [syscall_stack]  ← 模块的 Rust init()
dump_stack_lvl                   ← 打印栈
```

`rmmod` 的栈：

```
entry_SYSCALL_64_after_hwframe
do_syscall_64
__se_sys_delete_module           ← 176 号系统调用
cleanup_module+0x60/0xf00 [syscall_stack]  ← 模块的 Drop
dump_stack_lvl
```

寄存器里的 `ORIG_RAX` 就是系统调用号：

* `ORIG_RAX: 0x139` = 313 = `finit_module`
* `ORIG_RAX: 0xb0`  = 176 = `delete_module`

## 踩坑

* `module!` 的每一对 `key: value` 后面都必须有逗号，**最后一项也要**，否则报 `Reached end of token stream for Punct`；
* crate 根必须有 `//!` 文档注释，否则 `-Wmissing_docs` 会变成错误；
* `dump_stack()` 在 bindings 里，调用要包 `unsafe`；
* 日志里同一段栈出现两次，通常是 `dmesg` 把环形缓冲区又打印了一遍，不是执行了两次。
