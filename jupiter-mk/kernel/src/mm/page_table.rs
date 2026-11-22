//! Sv39 page tables (4 KiB pages only for now).

use alloc::vec::Vec;
use bitflags::bitflags;

use crate::mm::{PhysPageNum, VirtPageNum, alloc_frame, dealloc_frame, page_align_up};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PteFlags: usize {
        const V = 1 << 0; // valid
        const R = 1 << 1; // readable
        const W = 1 << 2; // writable
        const X = 1 << 3; // executable
        const U = 1 << 4; // user accessible
        const G = 1 << 5; // global
        const A = 1 << 6; // accessed
        const D = 1 << 7; // dirty
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn new(ppn: PhysPageNum, flags: PteFlags) -> Self {
        Self {
            bits: (ppn.0 << 10) | flags.bits(),
        }
    }

    /// Leaf PTE with Accessed/Dirty preset. Non-leaf table pointers must not
    /// carry A/D, so they keep using [`Self::new`].
    pub fn new_leaf(ppn: PhysPageNum, flags: PteFlags) -> Self {
        Self {
            bits: (ppn.0 << 10) | flags.bits() | PteFlags::A.bits() | PteFlags::D.bits(),
        }
    }

    pub fn ppn(self) -> PhysPageNum {
        PhysPageNum((self.bits >> 10) & ((1usize << 44) - 1))
    }

    pub fn flags(self) -> PteFlags {
        PteFlags::from_bits_truncate(self.bits & 0xff)
    }

    pub fn is_valid(self) -> bool {
        self.flags().contains(PteFlags::V)
    }
}

/// A frame owned by a page-table structure.
struct TableFrame {
    ppn: PhysPageNum,
}

impl TableFrame {
    fn alloc() -> Self {
        let ppn = alloc_frame().expect("out of frames for page table");
        Self { ppn }
    }
}

impl Drop for TableFrame {
    fn drop(&mut self) {
        dealloc_frame(self.ppn);
    }
}

/// A Sv39 page table. The root and all intermediate tables are owned by this
/// object and are returned to the frame allocator when it is dropped.
pub struct PageTable {
    root: TableFrame,
    tables: Vec<TableFrame>,
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            root: TableFrame::alloc(),
            tables: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        // Sv39 mode (8) in the top bits, then the root PPN.
        (8usize << 60) | self.root.ppn.0
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        let indexes = vpn.indexes();
        let mut ppn = self.root.ppn;

        for level in 0..3 {
            let pte = ppn.get_pte_array()[indexes[level]];
            if !pte.is_valid() {
                return None;
            }
            if level == 2 {
                return Some(pte);
            }
            ppn = pte.ppn();
        }
        None
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> &mut PageTableEntry {
        let indexes = vpn.indexes();
        let mut ppn = self.root.ppn;

        for level in 0..3 {
            let pte = &mut ppn.get_pte_array()[indexes[level]];
            if level == 2 {
                return pte;
            }

            if !pte.is_valid() {
                let frame = TableFrame::alloc();
                let next = frame.ppn;
                *pte = PageTableEntry::new(next, PteFlags::V);
                self.tables.push(frame);
                ppn = next;
            } else {
                ppn = pte.ppn();
            }
        }
        unreachable!()
    }

    /// Return (allocating if needed) the level-1 table for a root index.
    fn level1_table(&mut self, root_index: usize) -> PhysPageNum {
        let pte = &mut self.root.ppn.get_pte_array()[root_index];
        if pte.is_valid() {
            return pte.ppn();
        }
        let frame = TableFrame::alloc();
        let ppn = frame.ppn;
        *pte = PageTableEntry::new(ppn, PteFlags::V);
        self.tables.push(frame);
        ppn
    }

    /// Map one 2 MiB superpage. Requires 2 MiB alignment of both addresses.
    pub fn map_2m(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PteFlags) {
        assert!(
            vpn.0 % 512 == 0 && ppn.0 % 512 == 0,
            "2 MiB mapping is not aligned"
        );
        let indexes = vpn.indexes();
        let l1 = self.level1_table(indexes[0]);
        let pte = &mut l1.get_pte_array()[indexes[1]];
        assert!(!pte.is_valid(), "2 MiB VPN {:#x} is already mapped", vpn.0);
        *pte = PageTableEntry::new_leaf(ppn, flags | PteFlags::V);
    }

    /// Map one 4 KiB page. Panics if the VPN is already mapped.
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PteFlags) {
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid(), "VPN {:#x} is already mapped", vpn.0);
        *pte = PageTableEntry::new_leaf(ppn, flags | PteFlags::V);
    }

    /// Remove a mapping and return the old PTE. The physical frame is not
    /// freed here; the caller owns it.
    pub fn unmap(&mut self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        let pte = self.find_pte_create(vpn);
        if !pte.is_valid() {
            return None;
        }
        let old = *pte;
        *pte = PageTableEntry::empty();
        Some(old)
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).filter(|pte| pte.is_valid())
    }

    pub fn translate_va(&self, va: usize) -> Option<usize> {
        let vpn = VirtPageNum(va / crate::config::PAGE_SIZE);
        let pte = self.translate(vpn)?;
        Some(pte.ppn().0 * crate::config::PAGE_SIZE + va % crate::config::PAGE_SIZE)
    }

    /// Map a contiguous physical range with an identity mapping, using 2 MiB
    /// superpages where possible. Both ends must be 2 MiB aligned.
    pub fn map_identity_large(&mut self, start: usize, end: usize, flags: PteFlags) {
        const TWO_MIB: usize = 2 * 1024 * 1024;
        assert!(
            start % TWO_MIB == 0 && end % TWO_MIB == 0,
            "large identity range is not 2 MiB aligned"
        );
        let mut va = start;
        while va < end {
            let vpn = VirtPageNum(va / crate::config::PAGE_SIZE);
            let ppn = PhysPageNum(va / crate::config::PAGE_SIZE);
            self.map_2m(vpn, ppn, flags);
            va += TWO_MIB;
        }
    }

    /// Map a contiguous physical range with an identity mapping.
    pub fn map_identity(&mut self, start: usize, end: usize, flags: PteFlags) {
        let start = start & !(crate::config::PAGE_SIZE - 1);
        let end = page_align_up(end);
        let mut va = start;
        while va < end {
            let vpn = VirtPageNum(va / crate::config::PAGE_SIZE);
            let ppn = PhysPageNum(va / crate::config::PAGE_SIZE);
            self.map(vpn, ppn, flags);
            va += crate::config::PAGE_SIZE;
        }
    }
}
