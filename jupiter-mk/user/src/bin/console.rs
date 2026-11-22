//! Console server: the only task that touches the UART.
//!
//! It polls the 8250-compatible UART, forwards received bytes to the shell
//! over the input endpoint, and prints messages received on the console
//! request endpoint. This is a user-space driver; the kernel never programs
//! the UART.

#![no_std]
#![no_main]

use jupiter_user::*;

const LSR: usize = 5;
const LSR_DR: u8 = 1 << 0;
const LSR_THRE: u8 = 1 << 5;

#[inline(always)]
unsafe fn read8(base: usize, off: usize) -> u8 {
    unsafe { core::ptr::read_volatile((base + off) as *const u8) }
}

#[inline(always)]
unsafe fn write8(base: usize, off: usize, value: u8) {
    unsafe { core::ptr::write_volatile((base + off) as *mut u8, value) };
}

/// Wait for the transmit holding register, then write one byte.
#[inline(always)]
unsafe fn putc(base: usize, byte: u8) {
    while unsafe { read8(base, LSR) } & LSR_THRE == 0 {
        core::hint::spin_loop();
    }
    unsafe { write8(base, 0, byte) };
}

fn write_str(base: usize, s: &str) {
    for byte in s.bytes() {
        unsafe { putc(base, byte) };
    }
}

fn main() {
    let base = uart_base();
    if base == 0 {
        debug_write(b"console: no uart capability\n");
        return;
    }
    write_str(base, "[console] uart driver online\n");

    // Small input ring buffer. The shell may be busy printing, so never block
    // the console on a reply from the shell; buffer keystrokes instead.
    const INPUT_BUF: usize = 64;
    let mut input = [0u8; INPUT_BUF];
    let mut head = 0;
    let mut tail = 0;

    loop {
        // 1. UART -> input ring buffer
        unsafe {
            if read8(base, LSR) & LSR_DR != 0 {
                let ch = read8(base, 0);
                // Echo immediately so the terminal feels local.
                putc(base, ch);
                let next = (tail + 1) % INPUT_BUF;
                if next != head {
                    input[tail] = ch;
                    tail = next;
                }
            }
        }

        // 2. ring buffer -> shell (only when the shell is waiting)
        while head != tail {
            let msg = Message::new(TAG_INPUT, [input[head] as usize, 0, 0, 0]);
            if try_send(CAP_INPUT, &msg) < 0 {
                break;
            }
            head = (head + 1) % INPUT_BUF;
        }

        // 3. shell -> UART
        let mut msg = Message::default();
        while try_recv(CAP_CONSOLE, &mut msg) == 0 {
            let len = msg.words[0].min(MSG_WORDS * 8 - 8);
            for i in 0..len {
                unsafe { putc(base, msg.byte(i)) };
            }
        }

        yield_now();
    }
}

jupiter_user::entry!(main);
