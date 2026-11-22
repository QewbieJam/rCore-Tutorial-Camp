//! Trap entry and exit.
//!
//! The kernel uses a single global trap vector: every user page table also
//! contains the kernel identity map, so a trap from U-mode can execute kernel
//! code without switching `satp` or jumping through a user-space trampoline.
//!
//! `sscratch` always holds the top of the current task's kernel stack while
//! user code runs.

mod context;

use core::arch::global_asm;

use riscv::register::{scause, sstatus, stval, stvec, stvec::TrapMode};

use crate::sbi;
use crate::syscall;
use crate::task;

pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    fn __alltraps();
}

pub fn init() {
    unsafe {
        stvec::write(stvec::Stvec::new(
            __alltraps as *const () as usize,
            TrapMode::Direct,
        ));
        // Enable the supervisor timer interrupt. `sstatus.SIE` stays cleared in
        // S-mode; user TrapContexts set SPIE so the timer fires in U-mode.
        riscv::register::sie::set_stimer();
    }
    println!("[trap] stvec = {:#x}", __alltraps as *const () as usize);
}

/// Rust half of the trap entry sequence. Never returns: it always restores a
/// TrapContext and executes `sret`.
#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(cx: &mut TrapContext) -> ! {
    let scause = scause::read().bits();
    let is_interrupt = scause >> (usize::BITS - 1) != 0;
    let code = scause & !(1usize << (usize::BITS - 1));

    if is_interrupt {
        match code {
            5 => {
                // Supervisor timer: reprogram and reschedule.
                sbi::set_next_timer();
                task::preempt_current();
            }
            _ => {
                println!("[trap] unhandled interrupt {}", code);
            }
        }
    } else {
        match code {
            8 => {
                // Environment call from U-mode.
                cx.sepc += 4;
                let ret = syscall::dispatch(cx);
                cx.x[10] = ret as usize;
            }
            _ => {
                println!(
                    "[trap] fatal exception {} at sepc={:#x}, stval={:#x}",
                    code,
                    cx.sepc,
                    stval::read()
                );
                task::exit_current(-1);
            }
        }
    }

    trap_return(cx)
}

/// Restore `cx` and return to U-mode.
pub fn trap_return(cx: &mut TrapContext) -> ! {
    unsafe extern "C" {
        fn __restore(cx: *mut TrapContext) -> !;
    }
    unsafe { __restore(cx) }
}

/// Used by the very first `sret` into a user task: build a pointer to the
/// TrapContext at the top of the kernel stack.
pub fn user_sstatus_bits() -> usize {
    let mut bits = sstatus::read().bits();
    // Previous privilege = U, previous interrupt enable = 1.
    bits &= !(1 << 8);
    bits |= 1 << 5;
    bits &= !(1 << 1); // SIE is enabled by sret through SPIE
    bits
}
