#![no_std]
#![no_main]

#[no_mangle]
pub fn main() -> i32 {
    unix_print::unix_eprintln!("Starting alloc single threaded main");
    test_lib::run_tests();
    0
}
