#![no_std]
#![no_main]
#![allow(missing_docs)]

include!("mini.rs");

#[no_mangle]
pub extern "C" fn app_main() -> i32 {
    sys_reset_syscall_counts();
    0
}