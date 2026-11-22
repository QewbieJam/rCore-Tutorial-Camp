//! Physical frame allocator (bitmap, 4 KiB frames).
//!
//! This is intentionally simple and single-hart. The bitmap is replaced by a
//! per-NUMA-node buddy allocator when SMP support arrives.

use core::cmp::{max, min};

use spin::Mutex;

use crate::board;
use crate::config::{BITMAP_BYTES, MAX_FRAMES, MAX_MANAGED_MEMORY, PAGE_SIZE};
use crate::mm::{PhysPageNum, page_align_up};

unsafe extern "C" {
    fn ekernel();
}

static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator>> = Mutex::new(None);
static mut BITMAP: [u8; BITMAP_BYTES] = [0; BITMAP_BYTES];

struct FrameAllocator {
    bitmap: &'static mut [u8],
    start_ppn: usize,
    total: usize,
    free: usize,
    cursor: usize,
}

impl FrameAllocator {
    #[inline]
    fn is_free(&self, index: usize) -> bool {
        self.bitmap[index / 8] & (1 << (index % 8)) == 0
    }

    #[inline]
    fn set_used(&mut self, index: usize) {
        self.bitmap[index / 8] |= 1 << (index % 8);
    }

    #[inline]
    fn set_free(&mut self, index: usize) {
        self.bitmap[index / 8] &= !(1 << (index % 8));
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if self.free == 0 {
            return None;
        }

        let mut index = self.cursor;
        for _ in 0..self.total {
            if self.is_free(index) {
                self.set_used(index);
                self.free -= 1;
                self.cursor = (index + 1) % self.total;

                let ppn = PhysPageNum(self.start_ppn + index);
                // A freshly allocated frame must never leak another task's data.
                ppn.get_bytes_array().fill(0);
                return Some(ppn);
            }
            index = (index + 1) % self.total;
        }
        None
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        assert!(ppn.0 >= self.start_ppn);
        let index = ppn.0 - self.start_ppn;
        assert!(
            index < self.total,
            "frame {:#x} is outside managed RAM",
            ppn.0
        );
        assert!(!self.is_free(index), "double free of frame {:#x}", ppn.0);

        self.set_free(index);
        self.free += 1;
    }
}

pub fn init() {
    let memory_start = board::BOARD.memory_start;
    let memory_end = min(
        board::BOARD.memory_end,
        memory_start.saturating_add(MAX_MANAGED_MEMORY),
    );

    assert!(
        memory_start % PAGE_SIZE == 0,
        "RAM start is not page aligned"
    );
    assert!(memory_end > memory_start, "empty RAM range");

    let start_ppn = memory_start / PAGE_SIZE;
    let end_ppn = memory_end / PAGE_SIZE;
    let total = end_ppn - start_ppn;
    assert!(total <= MAX_FRAMES, "RAM exceeds bitmap capacity");

    let bitmap = unsafe { &mut *core::ptr::addr_of_mut!(BITMAP) };
    let used_bytes = total.div_ceil(8);
    bitmap[..used_bytes].fill(0xFF);

    let kernel_end = ekernel as *const () as usize;
    let first_free = max(page_align_up(kernel_end), memory_start) / PAGE_SIZE;
    assert!(
        first_free >= start_ppn && first_free <= end_ppn,
        "kernel end {:#x} is outside managed RAM",
        kernel_end
    );

    let mut allocator = FrameAllocator {
        bitmap,
        start_ppn,
        total,
        free: 0,
        cursor: first_free - start_ppn,
    };

    for ppn in first_free..end_ppn {
        allocator.set_free(ppn - start_ppn);
        allocator.free += 1;
    }

    println!(
        "[mm] frames: {:#x}..{:#x}, {} total, {} free, kernel_end={:#x}",
        memory_start, memory_end, total, allocator.free, kernel_end
    );

    *FRAME_ALLOCATOR.lock() = Some(allocator);
}

pub fn alloc_frame() -> Option<PhysPageNum> {
    FRAME_ALLOCATOR.lock().as_mut()?.alloc()
}

pub fn dealloc_frame(ppn: PhysPageNum) {
    if let Some(allocator) = FRAME_ALLOCATOR.lock().as_mut() {
        allocator.dealloc(ppn);
    } else {
        panic!("frame allocator is not initialized");
    }
}

pub fn free_frames() -> usize {
    FRAME_ALLOCATOR
        .lock()
        .as_ref()
        .map(|allocator| allocator.free)
        .unwrap_or(0)
}

pub fn total_frames() -> usize {
    FRAME_ALLOCATOR
        .lock()
        .as_ref()
        .map(|allocator| allocator.total)
        .unwrap_or(0)
}

pub fn run_tests() {
    println!("[mm] frame allocator test...");

    let before = free_frames();
    let a = alloc_frame().expect("out of frames");
    let b = alloc_frame().expect("out of frames");
    assert_ne!(a, b, "allocator returned the same frame twice");
    assert_eq!(free_frames(), before - 2);

    // Frames must be zeroed by the allocator.
    a.get_bytes_array()[0] = 0xaa;
    dealloc_frame(a);
    dealloc_frame(b);
    assert_eq!(free_frames(), before);

    let c = alloc_frame().expect("out of frames");
    assert_eq!(c.get_bytes_array()[0], 0, "allocated frame was not zeroed");
    dealloc_frame(c);

    println!(
        "[mm] frame test passed: {} total frames, {} free",
        total_frames(),
        free_frames()
    );
}
