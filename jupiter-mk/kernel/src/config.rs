//! Kernel-wide constants.

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BITS: usize = 12;

/// Static kernel heap. 16 MiB is enough for the MVP's kernel objects.
pub const KERNEL_HEAP_SIZE: usize = 16 * 1024 * 1024;

/// The bitmap can describe 2 GiB of physical memory (4 KiB pages).
/// Jupiter boards with more RAM are deliberately limited to the first 2 GiB
/// until the direct-map design is replaced by a full physical-memory map.
pub const MAX_MANAGED_MEMORY: usize = 2 * 1024 * 1024 * 1024;
pub const MAX_FRAMES: usize = MAX_MANAGED_MEMORY / PAGE_SIZE;
pub const BITMAP_BYTES: usize = MAX_FRAMES / 8;

pub const KERNEL_STACK_SIZE: usize = 64 * 1024;
