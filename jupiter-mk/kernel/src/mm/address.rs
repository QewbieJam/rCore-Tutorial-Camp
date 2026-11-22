//! Physical/virtual address newtypes.
//!
//! This is deliberately small; Sv39 page-table walkers arrive with the vspace
//! layer.

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl VirtPageNum {
    /// Sv39 three-level indexes, highest level first: VPN[2], VPN[1], VPN[0].
    /// The value is a page number, so the lowest index is `self.0 & 511`.
    pub fn indexes(self) -> [usize; 3] {
        [
            (self.0 >> 18) & 0x1ff,
            (self.0 >> 9) & 0x1ff,
            self.0 & 0x1ff,
        ]
    }
}

impl PhysPageNum {
    pub fn start_address(self) -> PhysAddr {
        PhysAddr(self.0 * PAGE_SIZE)
    }

    /// The frame contents. The caller is responsible for synchronization and
    /// for keeping the frame alive while the slice is in use.
    pub fn get_bytes_array(self) -> &'static mut [u8] {
        let pa = self.start_address().0;
        unsafe { core::slice::from_raw_parts_mut(pa as *mut u8, PAGE_SIZE) }
    }

    pub fn get_mut<T>(self) -> &'static mut T {
        let pa = self.start_address().0;
        unsafe { &mut *(pa as *mut T) }
    }

    /// View this frame as a 512-entry page table. Only valid when the frame
    /// actually holds page-table entries.
    pub fn get_pte_array(self) -> &'static mut [crate::mm::page_table::PageTableEntry; 512] {
        let pa = self.start_address().0;
        unsafe { &mut *(pa as *mut [crate::mm::page_table::PageTableEntry; 512]) }
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

pub fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

pub fn align_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

pub fn page_align_up(value: usize) -> usize {
    align_up(value, PAGE_SIZE)
}

pub fn page_align_down(value: usize) -> usize {
    align_down(value, 1 << PAGE_SIZE_BITS)
}
