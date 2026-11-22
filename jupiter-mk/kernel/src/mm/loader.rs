//! Minimal ELF64 loader for statically linked user programs.
//!
//! Only little-endian RV64 `ET_EXEC` files with `PT_LOAD` segments are
//! supported, which is exactly what the user workspace produces.

use crate::config::PAGE_SIZE;
use crate::mm::{VirtPageNum, page_align_down, page_align_up, vspace::MemorySet};

const PT_LOAD: u32 = 1;

fn u16_at(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

fn u32_at(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn u64_at(data: &[u8], off: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(bytes)
}

/// Load an ELF image into `space` and return its entry point.
pub fn load_elf(space: &mut MemorySet, data: &[u8]) -> usize {
    assert!(data.len() >= 64, "user image is too small to be an ELF");
    assert_eq!(&data[0..4], b"\x7fELF", "user image is not an ELF");
    assert_eq!(data[4], 2, "user image must be ELF64");
    assert_eq!(data[5], 1, "user image must be little endian");
    assert_eq!(u16_at(data, 18), 243, "user image is not RISC-V");

    let entry = u64_at(data, 24) as usize;
    let phoff = u64_at(data, 32) as usize;
    let phentsize = u16_at(data, 54) as usize;
    let phnum = u16_at(data, 56) as usize;
    assert!(phentsize >= 56, "unexpected program header size");

    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        assert!(ph + 56 <= data.len(), "program header out of bounds");
        if u32_at(data, ph) != PT_LOAD {
            continue;
        }

        let offset = u64_at(data, ph + 8) as usize;
        let vaddr = u64_at(data, ph + 16) as usize;
        let filesz = u64_at(data, ph + 32) as usize;
        let memsz = u64_at(data, ph + 40) as usize;
        if memsz == 0 {
            continue;
        }

        let seg_start = page_align_down(vaddr);
        let seg_end = page_align_up(vaddr + memsz);
        for va in (seg_start..seg_end).step_by(PAGE_SIZE) {
            let vpn = VirtPageNum(va / PAGE_SIZE);
            let ppn = space.map_user_page(vpn);
            let page = ppn.get_bytes_array();

            // Copy the part of this page covered by file data. Frames are
            // zeroed when allocated, so BSS needs no extra work.
            let copy_start = core::cmp::max(va, vaddr);
            let copy_end = core::cmp::min(va + PAGE_SIZE, vaddr + filesz);
            if copy_start < copy_end {
                let len = copy_end - copy_start;
                let src = offset + (copy_start - vaddr);
                assert!(src + len <= data.len(), "ELF segment exceeds file");
                page[copy_start - va..copy_start - va + len].copy_from_slice(&data[src..src + len]);
            }
        }
    }

    entry
}
