#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

#[repr(C)]
struct TaskInfo {
    tid: u32,
    pid: u32,
    comm: [u8; 16],
}

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
    let mut ti = TaskInfo {
        tid: 0,
        pid: 0,
        comm: [0u8; 16],
    };
    let r = unsafe {
        syscall3(463, &mut ti as *mut TaskInfo as usize, core::mem::size_of::<TaskInfo>(), 0,)
    };

    print("get_taskinfo ret=");
    print_u(r as u64);
    print("\n");

    print("tid=");
    print_u(ti.tid as u64);
    print(" pid=");
    print_u(ti.pid as u64);
    print(" comm=");
    print_comm(&ti.comm);
    print("\n");

    unsafe { syscall4(142, 0xfee1dead, 672274793, 0x4321fedc, 0); }
    0
}

fn print(s: &str) {
    unsafe { syscall3(64, 1, s.as_ptr() as usize, s.len()); }
}

fn print_u(mut v: u64) {
    if v == 0 {
        print("0");
        return;
    }
    let mut buf = [0u8; 24];
    let mut i = buf.len();
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    unsafe { syscall3(64, 1, buf[i..].as_ptr() as usize, buf.len() - i); }
}

fn print_comm(comm: &[u8; 16]) {
    let n = comm.iter().position(|&c| c ==0 ).unwrap_or(16);
    unsafe { syscall3(64, 1, comm.as_ptr() as usize, n); }
}
