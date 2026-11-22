//! Minimal SBI client.
//!
//! The kernel uses SBI only for early boot services:
//! - debug console (DBCN, with legacy putchar fallback)
//! - timer programming
//! - system reset
//!
//! Real device drivers live in user space.

use core::arch::asm;

const SBI_LEGACY_CONSOLE_PUTCHAR: usize = 0x01;
const SBI_LEGACY_SHUTDOWN: usize = 0x08;
const SBI_TIME: usize = 0x5449_4D45; // "TIME"
const SBI_SRST: usize = 0x5352_5354; // "SRST"
const SBI_DBCN: usize = 0x4442_434E; // "DBCN"

#[derive(Clone, Copy)]
pub struct SbiRet {
    pub error: usize,
    #[allow(dead_code)]
    pub value: usize,
}

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
            options(nostack)
        );
    }
    SbiRet { error, value }
}

/// Write a byte to the SBI console.
pub fn console_putchar(c: u8) {
    let _ = sbi_call(SBI_LEGACY_CONSOLE_PUTCHAR, 0, c as usize, 0, 0);
}

/// Write a string through the SBI DBCN extension.
///
/// The buffer is currently in the kernel's identity map, so its virtual
/// address is also its physical address.
pub fn console_write(s: &str) {
    if s.is_empty() {
        return;
    }
    let ret = sbi_call(
        SBI_DBCN,
        0,
        s.len(),
        s.as_ptr() as usize,
        0, // high half of the physical address
    );
    if ret.error != 0 {
        for b in s.bytes() {
            console_putchar(b);
        }
    }
}

/// Program the next supervisor timer interrupt.
pub fn set_timer(stime: usize) {
    let _ = sbi_call(SBI_TIME, 0, stime, 0, 0);
}

/// Arm the timer for the next scheduler tick.
pub fn set_next_timer() {
    let now = riscv::register::time::read();
    let interval = crate::board::BOARD.timebase_hz / 100; // 10 ms
    set_timer(now + interval);
}

/// Power off the machine (or park the hart if SRST is unavailable).
pub fn shutdown(failure: bool) -> ! {
    let reason = if failure { 1 } else { 0 };
    let _ = sbi_call(SBI_SRST, 0, 0, reason, 0);
    let _ = sbi_call(SBI_LEGACY_SHUTDOWN, 0, reason, 0, 0);

    loop {
        unsafe {
            asm!("wfi", options(nomem, nostack));
        }
    }
}
