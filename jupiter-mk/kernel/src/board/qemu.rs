//! QEMU `virt` machine constants.

use super::BoardInfo;

pub const BOARD: BoardInfo = BoardInfo {
    name: "qemu-virt",
    uart_base: 0x1000_0000,
    plic_base: 0x0c00_0000,
    clint_base: 0x0200_0000,
    timebase_hz: 10_000_000,
    memory_start: 0x8000_0000,
    memory_end: 0x8800_0000, // 128 MiB, the default QEMU RAM size
    kernel_load: 0x8020_0000,
};
