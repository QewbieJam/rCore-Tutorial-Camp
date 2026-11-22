//! System call dispatch.
//!
//! Syscalls are intentionally limited to mechanisms a microkernel must own:
//! task lifecycle, memory/device bootstrap, and IPC. Console output is a
//! temporary debug convenience; normal output goes through the console server.

use crate::board;
use crate::sbi;
use crate::task;
use crate::trap::TrapContext;

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

pub fn dispatch(cx: &mut TrapContext) -> isize {
    let id = cx.x[17]; // a7
    let a0 = cx.x[10];
    let a1 = cx.x[11];

    match id {
        SYS_EXIT => task::exit_current(a0 as isize),
        SYS_YIELD => {
            task::preempt_current();
            0
        }
        SYS_GETPID => task::current_pid() as isize,
        SYS_DEBUG_WRITE => debug_write(a0, a1),
        SYS_SEND => match task::read_user_message(a1) {
            Some(msg) => task::send(a0, msg),
            None => -1,
        },
        SYS_RECV => match task::recv(a0) {
            Some(msg) if task::write_user_message(a1, &msg) => 0,
            _ => -1,
        },
        SYS_TRY_RECV => match task::try_recv(a0) {
            Some(msg) if task::write_user_message(a1, &msg) => 0,
            _ => -1,
        },
        SYS_SPAWN => task::spawn(a0),
        SYS_UART_BASE => board::BOARD.uart_base as isize,
        SYS_TRY_SEND => match task::read_user_message(a1) {
            Some(msg) => task::try_send(a0, msg),
            None => -1,
        },
        SYS_MEMINFO => crate::mm::free_frames() as isize,
        _ => {
            println!(
                "[syscall] pid {}: unknown syscall {}",
                task::current_pid(),
                id
            );
            -1
        }
    }
}

/// Temporary kernel debug output. Not used by the console server.
fn debug_write(ptr: usize, len: usize) -> isize {
    let mut buf = [0u8; 256];
    let len = len.min(buf.len());
    if !task::read_user_bytes(ptr, len, &mut buf) {
        return -1;
    }
    for byte in &buf[..len] {
        sbi::console_putchar(*byte);
    }
    len as isize
}
