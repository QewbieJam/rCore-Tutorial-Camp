//! Early kernel console (SBI DBCN).
//!
//! This is a boot/debug console only. Once user mode is available, the UART
//! is owned by a user-space console server and applications talk to it over
//! IPC.

use core::fmt::{self, Write};

pub fn init() {}

pub fn _print(args: fmt::Arguments) {
    let mut writer = Console;
    let _ = writer.write_fmt(args);
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        crate::sbi::console_write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::console::_print(format_args!("\r\n"))
    };
    ($($arg:tt)*) => {{
        $crate::console::_print(format_args!($($arg)*));
        $crate::console::_print(format_args!("\r\n"));
    }};
}
