//! Trap context: 32 general-purpose registers plus sstatus/sepc.

use core::mem::size_of;

/// Layout shared with `trap.S`.
///
/// ```text
/// x0 .. x31   offsets 0   .. 31*8
/// sstatus     offset 32*8
/// sepc        offset 33*8
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

const _: () = assert!(size_of::<TrapContext>() == 34 * 8);

impl TrapContext {
    /// Build the initial context for entering a user program.
    pub fn new_user(entry: usize, user_sp: usize) -> Self {
        let mut cx = Self {
            x: [0; 32],
            sstatus: crate::trap::user_sstatus_bits(),
            sepc: entry,
        };
        cx.x[2] = user_sp;
        cx
    }
}
