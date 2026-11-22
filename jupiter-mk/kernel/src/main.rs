//! jupiter-mk: a small RISC-V microkernel for QEMU virt and Milk-V Jupiter.
//!
//! The kernel owns traps, physical memory, Sv39 address spaces, tasks,
//! scheduling and synchronous IPC. Device drivers, the console and the shell
//! run in user space.

#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
mod console;
mod board;
mod config;
mod ipc;
mod lang_items;
mod mm;
mod sbi;
mod syscall;
mod task;
mod trap;
mod user_images;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

/// Rust entry point called by `entry.asm`.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hartid: usize, dtb: usize) -> ! {
    console::init();

    println!();
    println!("==============================================");
    println!("  jupiter-mk / RISC-V microkernel");
    println!("  board : {}", board::BOARD.name);
    println!("  hart  : {}", hartid);
    println!("  dtb   : {:#x}", dtb);
    println!(
        "  ram   : {:#x}..{:#x}",
        board::BOARD.memory_start,
        board::BOARD.memory_end
    );
    println!("  uart  : {:#x}", board::BOARD.uart_base);
    println!("  load  : {:#x}", board::BOARD.kernel_load);
    println!("==============================================");

    mm::init();
    mm::run_tests();

    trap::init();
    sbi::set_next_timer();
    task::init();

    println!();
    println!("[m3] entering user mode: console server + shell");
    println!();
    task::run_tasks();
}
