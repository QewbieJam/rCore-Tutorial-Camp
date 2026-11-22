//! Memory management: physical frames, kernel heap, Sv39 address spaces and
//! the user ELF loader.

mod address;
mod frame;
mod heap;
pub mod loader;
pub mod page_table;
mod vspace;

pub use address::{PhysPageNum, VirtPageNum, page_align_down, page_align_up};
pub use frame::{alloc_frame, dealloc_frame, free_frames};
pub use vspace::{MemorySet, USER_STACK_TOP, kernel_token};

pub fn init() {
    heap::init();
    frame::init();
    vspace::init();
}

pub fn run_tests() {
    heap::run_tests();
    frame::run_tests();
    vspace::run_tests();
}
