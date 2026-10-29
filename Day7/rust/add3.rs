#![no_std]
#![no_main]

extern "C" {
    fn print_int(x: i64);
}

#[no_mangle]
pub extern "C" fn add3(a: i64, b: i64, c: i64) -> i64 {
    a + b + c
}

#[no_mangle]
pub extern "C" fn main() {
    let x = add3(1, 2, 3);
    unsafe { print_int(x) };
}
