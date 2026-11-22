//! User-space runtime for jupiter-mk.
//!
//! The only kernel interface is the syscall ABI: task lifecycle, memory
//! bootstrap and synchronous IPC. Console output is a message to capability
//! slot 0, which is the console server endpoint.

#![no_std]

use core::arch::asm;

pub const SYS_EXIT: usize = 0;
pub const SYS_YIELD: usize = 1;
pub const SYS_GETPID: usize = 2;
pub const SYS_DEBUG_WRITE: usize = 3;
pub const SYS_SEND: usize = 4;
pub const SYS_RECV: usize = 5;
pub const SYS_TRY_RECV: usize = 6;
pub const SYS_SPAWN: usize = 7;
pub const SYS_UART_BASE: usize = 8;
pub const SYS_TRY_SEND: usize = 9;
pub const SYS_MEMINFO: usize = 10;

pub const CAP_CONSOLE: usize = 0;
pub const CAP_INPUT: usize = 1;

pub const TAG_WRITE: usize = 1;
pub const TAG_INPUT: usize = 2;

pub const MSG_WORDS: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Message {
    pub tag: usize,
    pub words: [usize; MSG_WORDS],
}

impl Message {
    pub const fn new(tag: usize, words: [usize; MSG_WORDS]) -> Self {
        Self { tag, words }
    }

    pub fn from_bytes(tag: usize, bytes: &[u8]) -> Self {
        let mut msg = Self {
            tag,
            words: [0; MSG_WORDS],
        };
        msg.words[0] = bytes.len();
        for (i, byte) in bytes.iter().take(MSG_WORDS * 8 - 8).enumerate() {
            let word = 1 + i / 8;
            msg.words[word] |= (*byte as usize) << (8 * (i % 8));
        }
        msg
    }

    pub fn byte(&self, index: usize) -> u8 {
        let byte_index = index + 8;
        let word = byte_index / 8;
        let shift = 8 * (byte_index % 8);
        if word < MSG_WORDS {
            (self.words[word] >> shift) as u8
        } else {
            0
        }
    }
}

#[inline(always)]
pub fn syscall(id: usize, a0: usize, a1: usize, a2: usize) -> isize {
    let ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") a0 => ret,
            in("a1") a1,
            in("a2") a2,
            in("a7") id,
            options(nostack),
        );
    }
    ret
}

pub fn exit(code: i32) -> ! {
    syscall(SYS_EXIT, code as usize, 0, 0);
    loop {}
}

pub fn yield_now() {
    syscall(SYS_YIELD, 0, 0, 0);
}

pub fn getpid() -> usize {
    syscall(SYS_GETPID, 0, 0, 0) as usize
}

pub fn debug_write(bytes: &[u8]) -> isize {
    syscall(SYS_DEBUG_WRITE, bytes.as_ptr() as usize, bytes.len(), 0)
}

pub fn send(cap: usize, msg: &Message) -> isize {
    syscall(SYS_SEND, cap, msg as *const Message as usize, 0)
}

pub fn recv(cap: usize, msg: &mut Message) -> isize {
    syscall(SYS_RECV, cap, msg as *mut Message as usize, 0)
}

pub fn try_recv(cap: usize, msg: &mut Message) -> isize {
    syscall(SYS_TRY_RECV, cap, msg as *mut Message as usize, 0)
}

pub fn try_send(cap: usize, msg: &Message) -> isize {
    syscall(SYS_TRY_SEND, cap, msg as *const Message as usize, 0)
}

pub fn spawn(image: usize) -> isize {
    syscall(SYS_SPAWN, image, 0, 0)
}

pub fn uart_base() -> usize {
    syscall(SYS_UART_BASE, 0, 0, 0) as usize
}

/// Free physical frames currently available to the kernel.
pub fn meminfo() -> usize {
    syscall(SYS_MEMINFO, 0, 0, 0) as usize
}

/// Send a byte string to the console server in 24-byte messages.
pub fn print_bytes(bytes: &[u8]) {
    for chunk in bytes.chunks(MSG_WORDS * 8 - 8) {
        let msg = Message::from_bytes(TAG_WRITE, chunk);
        if send(CAP_CONSOLE, &msg) < 0 {
            // No console capability: fall back to the debug syscall.
            debug_write(chunk);
        }
    }
}

pub fn print(s: &str) {
    print_bytes(s.as_bytes());
}

/// Print an unsigned decimal number without allocation.
pub fn print_usize(mut value: usize) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    if value == 0 {
        print("0");
        return;
    }
    while value > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    print_bytes(&buf[i..]);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    debug_write(b"\n[user panic]\n");
    exit(1)
}

#[macro_export]
macro_rules! entry {
    ($main:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn _start() -> ! {
            $main();
            $crate::exit(0)
        }
    };
}
