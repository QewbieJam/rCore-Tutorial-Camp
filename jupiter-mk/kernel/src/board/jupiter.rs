//! Milk-V Jupiter v1.1 / SpacemiT K1 constants.
//!
//! Values are taken from the upstream K1 device tree:
//! - UART0: 0xd4017000, 8250/XScale, 115200 8n1
//! - PLIC : 0xe0000000
//! - CLINT: 0xe4000000
//! - timebase: 24 MHz
//!
//! U-Boot loads the kernel ELF at 0x2000_0000. Only the first 2 GiB of RAM
//! are managed by the current bitmap allocator; the rest is ignored until the
//! physical-memory map is upgraded.

use super::BoardInfo;

pub const BOARD: BoardInfo = BoardInfo {
    name: "milk-v-jupiter-v1.1 (spacemit k1)",
    uart_base: 0xd401_7000,
    plic_base: 0xe000_0000,
    clint_base: 0xe400_0000,
    timebase_hz: 24_000_000,
    memory_start: 0x0000_0000,
    memory_end: 0x8000_0000, // first 2 GiB
    kernel_load: 0x2000_0000,
};
