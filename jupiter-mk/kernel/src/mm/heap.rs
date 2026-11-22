//! Kernel heap backed by a static BSS region.
//!
//! A future milestone will replace the static heap with frames mapped into a
//! kernel direct-map window.

use alloc::boxed::Box;
use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
use core::ptr::addr_of_mut;

use crate::config::KERNEL_HEAP_SIZE;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(addr_of_mut!(HEAP_SPACE) as usize, KERNEL_HEAP_SIZE);
    }
    println!("[mm] kernel heap: {} KiB", KERNEL_HEAP_SIZE / 1024);
}

pub fn run_tests() {
    println!("[mm] heap test...");

    let boxed = Box::new(0x1234_5678usize);
    assert_eq!(*boxed, 0x1234_5678);
    drop(boxed);

    let mut values = Vec::new();
    for i in 0..1024 {
        values.push(i);
    }
    for (i, value) in values.iter().enumerate() {
        assert_eq!(*value, i);
    }
    drop(values);

    println!("[mm] heap test passed");
}
