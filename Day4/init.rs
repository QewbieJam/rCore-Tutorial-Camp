#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

global_asm!(
    ".global _start",
    "_start:",
    "mv s0, zero",          // 栈指针 = 0, 标记最外层
    "andi sp, sp, -16",     // 栈 16 字节对齐
    "call app_main",        // 调 Rust 函数
    "li a7, 93",            // __NR_exit = 93, exit 系统调用号
    "ecall",
);

/// 系统调用的封装
#[inline(always)]
unsafe fn syscall3(n: usize, a: usize, b: usize, c: usize) -> isize {
    let ret: isize;
    asm!(
        "ecall",
        inlateout("a0") a as isize => ret,      // a0即是参数，也是返回值

        in("a1") b,
        in("a2") c,
        in("a7") n,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn syscall4(n: usize, a: usize, b:usize, c: usize, d: usize) -> isize {
    let ret: isize;
    asm!(
        "ecall",
        inlateout("a0") a as isize => ret,

        in("a1") b, in("a2") c, in("a3") d, in("a7") n,
        options(nostack)
    );
    ret
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop{}
}

#[no_mangle]
pub extern "C" fn app_main() -> i32 {
    let msg = b"hello riscv\n";
    unsafe { syscall3(64, 1, msg.as_ptr() as usize, msg.len()); }
    unsafe { syscall4(142, 0xfee1dead, 672274793, 0x4321fedc, 0); }
    0
}
