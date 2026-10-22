// sys_write, sys_exit, sys_getpid, sys_get_taskinfo, sys_reset_syscall_counts,
// print, print_u, print_bytes, print_comm,
// memcpy, memmove, memset, bcmp

use core::arch::{asm, global_asm};

#[repr(C)]
pub struct TaskInfo {
    pub tid: u32,
    pub pid: u32,
    pub comm: [u8; 16],
}

#[inline(always)]
unsafe fn syscall3(n: usize, a: usize, b: usize, c: usize) -> isize {
    let ret: isize;
    asm!(
        "syscall",
        inlateout("rax") n as isize => ret,
        in("rdi") a,
        in("rsi") b,
        in("rdx") c,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

pub fn sys_write(fd: i32, buf: &[u8]) -> isize {
    unsafe {
        syscall3(1, fd as usize, buf.as_ptr() as usize, buf.len())
    }
}

pub fn sys_exit(code: i32) -> ! {
    unsafe {
        syscall3(60, code as usize, 0,0)
    };
    loop {}
}

pub fn sys_getpid() -> isize {
    unsafe { syscall3(39, 0, 0, 0) }
}

pub fn sys_get_taskinfo(ti: &mut TaskInfo) -> isize {
    unsafe {
        syscall3(
            463,
            ti as *mut TaskInfo as usize,
            core::mem::size_of::<TaskInfo>(),
            0,
        )
    }
}

pub fn sys_get_syscall_count(nr: u32) -> isize {
    unsafe { syscall3(464, nr as usize, 0, 0) }
}

pub fn sys_reset_syscall_counts() -> isize {
    unsafe { syscall3(465, 0, 0, 0) }
}

pub fn print(s: &str) {
    sys_write(1, s.as_bytes());

}

pub fn print_u(mut v: u64){
    let mut buf = [0u8; 24];
    let mut i = buf.len() - 1;
    if v == 0 {
        buf[i] = b'0';
        print_bytes(&buf[i..]);
        return;
    }
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    print_bytes(&buf[i..]);
}

pub fn print_bytes(b: &[u8]) {
    sys_write(1, b);
}

pub fn print_comm(comm: &[u8; 16]) {
    let n = comm.iter().position(|&c| c == 0).unwrap_or(16);
    print_bytes(&comm[..n]);
}

global_asm!(
    ".global _start",
    "_start:",
    "xor rbp, rbp",
    "and rsp, -16",
    "call app_main",
    "mov edi, eax",
    "mov eax, 60",
    "syscall",
);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    print("PANIC\n");
    sys_exit(127)
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n:usize) -> *mut u8 {
    if(dst as usize) < (src as usize) {
        let mut i = 0;
        while i < n {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    } else {
        let mut i = n;
        while i > 0 {
            i -= 1;
            *dst.add(i) = *src.add(i);
        }
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dst.add(i) = c as u8;
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let x = *a.add(i);
        let y = *b.add(i);
        if x != y {
            return x as i32 - y as i32;
        }
        i+=1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    memcmp(a, b, n)
}


