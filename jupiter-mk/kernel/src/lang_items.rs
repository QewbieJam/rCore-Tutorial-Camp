//! Panic and allocator error handling.

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("[kernel panic]");
    if let Some(location) = info.location() {
        println!("  at {}:{}", location.file(), location.line());
    }
    println!("  {}", info.message());
    crate::sbi::shutdown(true);
}
