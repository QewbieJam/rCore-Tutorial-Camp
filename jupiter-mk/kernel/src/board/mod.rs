//! Board abstraction.
//!
//! Only the few constants needed by the early kernel live here. Drivers are
//! user-space servers, so the kernel board layer deliberately does not contain
//! an UART or block-device driver.

#[cfg(feature = "board_jupiter")]
mod jupiter;
#[cfg(feature = "board_qemu")]
mod qemu;

#[cfg(all(feature = "board_qemu", feature = "board_jupiter"))]
compile_error!("enable exactly one board feature");

#[cfg(not(any(feature = "board_qemu", feature = "board_jupiter")))]
compile_error!("enable one of: board_qemu, board_jupiter");

#[cfg(feature = "board_jupiter")]
pub use jupiter::BOARD;
#[cfg(feature = "board_qemu")]
pub use qemu::BOARD;

/// Static description of the machine we are running on.
#[derive(Clone, Copy)]
pub struct BoardInfo {
    pub name: &'static str,
    pub uart_base: usize,
    pub plic_base: usize,
    pub clint_base: usize,
    pub timebase_hz: usize,
    pub memory_start: usize,
    pub memory_end: usize,
    pub kernel_load: usize,
}
