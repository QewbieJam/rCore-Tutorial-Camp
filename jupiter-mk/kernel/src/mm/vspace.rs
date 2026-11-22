//! Kernel and user address spaces.
//!
//! The kernel uses an identity map (`VA == PA`) for all managed RAM. Every
//! user page table includes that same supervisor-only identity map, so traps
//! and context switches never need to change `satp` just to run kernel code.
//!
//! User mappings live at [`USER_BASE`] (4 GiB) and above, outside the first
//! 2 GiB of RAM the bitmap allocator manages.

use alloc::vec::Vec;
use core::arch::asm;

use riscv::register::satp;
use spin::Mutex;

use crate::board;
use crate::config::{MAX_MANAGED_MEMORY, PAGE_SIZE};
use crate::mm::{
    PhysPageNum, VirtPageNum, alloc_frame, dealloc_frame,
    page_table::{PageTable, PteFlags},
};

/// First virtual address handed to user programs.
pub const USER_BASE: usize = 0x1_0000_0000;
/// User stack top. The stack is mapped downwards from here.
pub const USER_STACK_TOP: usize = USER_BASE + 0x0100_0000;
pub const USER_STACK_PAGES: usize = 8;

pub struct MemorySet {
    pub page_table: PageTable,
    /// Frames owned by user mappings. Page-table frames are owned by
    /// `page_table` itself.
    frames: Vec<PhysPageNum>,
}

static KERNEL_SPACE: Mutex<Option<MemorySet>> = Mutex::new(None);

impl MemorySet {
    fn bare() -> Self {
        Self {
            page_table: PageTable::new(),
            frames: Vec::new(),
        }
    }

    /// Address space for the kernel itself (RAM plus the MMIO pages the
    /// kernel uses during boot).
    pub fn new_kernel() -> Self {
        let mut space = Self::bare();
        space.map_kernel_ram();
        space.map_identity_user(
            board::BOARD.uart_base,
            board::BOARD.uart_base + PAGE_SIZE,
            PteFlags::R | PteFlags::W,
        );
        space.map_identity_user(
            board::BOARD.plic_base,
            board::BOARD.plic_base + PAGE_SIZE,
            PteFlags::R | PteFlags::W,
        );
        space.map_identity_user(
            board::BOARD.clint_base,
            board::BOARD.clint_base + PAGE_SIZE,
            PteFlags::R | PteFlags::W,
        );
        space
    }

    /// Address space for a user task: kernel RAM identity map with supervisor
    /// permissions, plus empty user mappings.
    pub fn new_user() -> Self {
        let mut space = Self::bare();
        space.map_kernel_ram();
        space
    }

    fn map_kernel_ram(&mut self) {
        let start = board::BOARD.memory_start;
        let end = core::cmp::min(
            board::BOARD.memory_end,
            start.saturating_add(MAX_MANAGED_MEMORY),
        );

        // The kernel identity map is the bulk of every user page table, so
        // use 2 MiB superpages. Both board ranges are 2 MiB aligned; keep a
        // 4 KiB fallback for the general case.
        const TWO_MIB: usize = 2 * 1024 * 1024;
        let large_start = crate::mm::page_align_up(start) & !(TWO_MIB - 1);
        let large_end = end & !(TWO_MIB - 1);
        if large_start < large_end {
            if large_start > start {
                self.page_table.map_identity(
                    start,
                    large_start,
                    PteFlags::R | PteFlags::W | PteFlags::X,
                );
            }
            self.page_table.map_identity_large(
                large_start,
                large_end,
                PteFlags::R | PteFlags::W | PteFlags::X,
            );
            if large_end < end {
                self.page_table.map_identity(
                    large_end,
                    end,
                    PteFlags::R | PteFlags::W | PteFlags::X,
                );
            }
        } else {
            self.page_table
                .map_identity(start, end, PteFlags::R | PteFlags::W | PteFlags::X);
        }
    }

    /// Identity map an MMIO range. `PteFlags::U` makes it reachable from
    /// U-mode (`console` uses this for the UART page).
    pub fn map_identity_user(&mut self, start: usize, end: usize, flags: PteFlags) {
        self.page_table.map_identity(start, end, flags);
    }

    /// Map one user page, allocating a zeroed frame if needed. Returns the
    /// backing physical page.
    pub fn map_user_page(&mut self, vpn: VirtPageNum) -> PhysPageNum {
        if let Some(pte) = self.page_table.translate(vpn) {
            return pte.ppn();
        }
        let ppn = alloc_frame().expect("out of memory: user page");
        self.page_table.map(
            vpn,
            ppn,
            PteFlags::R | PteFlags::W | PteFlags::X | PteFlags::U,
        );
        self.frames.push(ppn);
        ppn
    }

    /// Reserve the initial user stack.
    pub fn map_user_stack(&mut self) {
        let top_vpn = USER_STACK_TOP / PAGE_SIZE;
        for i in 1..=USER_STACK_PAGES {
            self.map_user_page(VirtPageNum(top_vpn - i));
        }
    }

    pub fn translate_va(&self, va: usize) -> Option<usize> {
        self.page_table.translate_va(va)
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    pub fn activate(&self) {
        unsafe {
            satp::write(satp::Satp::from_bits(self.page_table.token()));
            asm!("sfence.vma");
        }
    }
}

impl Drop for MemorySet {
    fn drop(&mut self) {
        for ppn in self.frames.drain(..) {
            dealloc_frame(ppn);
        }
    }
}

pub fn init() {
    let space = MemorySet::new_kernel();
    space.activate();
    println!(
        "[mm] Sv39 kernel identity map {:#x}..{:#x}, user base {:#x}",
        board::BOARD.memory_start,
        core::cmp::min(
            board::BOARD.memory_end,
            board::BOARD.memory_start.saturating_add(MAX_MANAGED_MEMORY)
        ),
        USER_BASE
    );
    *KERNEL_SPACE.lock() = Some(space);
}

pub fn kernel_token() -> usize {
    KERNEL_SPACE
        .lock()
        .as_ref()
        .map(MemorySet::token)
        .unwrap_or(0)
}

/// Map one page into the kernel address space. Used by allocator self-tests.
pub fn map_kernel_page(vpn: VirtPageNum, ppn: PhysPageNum, flags: PteFlags) {
    let mut space = KERNEL_SPACE.lock();
    space
        .as_mut()
        .expect("kernel space is not initialized")
        .page_table
        .map(vpn, ppn, flags);
    drop(space);
    unsafe {
        asm!("sfence.vma");
    }
}

pub fn unmap_kernel_page(vpn: VirtPageNum) -> Option<PhysPageNum> {
    let mut space = KERNEL_SPACE.lock();
    let old = space
        .as_mut()
        .expect("kernel space is not initialized")
        .page_table
        .unmap(vpn);
    drop(space);
    unsafe {
        asm!("sfence.vma");
    }
    old.map(|pte| pte.ppn())
}

pub fn run_tests() {
    println!("[mm] Sv39 mapping test...");

    let ppn = alloc_frame().expect("out of frames");
    let vpn = VirtPageNum(USER_BASE / PAGE_SIZE);
    map_kernel_page(vpn, ppn, PteFlags::R | PteFlags::W);

    let va = USER_BASE as *mut u8;
    unsafe {
        va.write_volatile(0x5a);
        assert_eq!(va.read_volatile(), 0x5a, "Sv39 read/write failed");
    }

    let old = unmap_kernel_page(vpn).expect("mapping disappeared");
    assert_eq!(old, ppn);
    dealloc_frame(ppn);

    println!("[mm] Sv39 test passed");
}
