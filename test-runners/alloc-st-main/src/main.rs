#![no_std]
#![no_main]

#[no_mangle]
pub fn main() -> i32 {
    tiny_std::eprintln!("Starting alloc single threaded main");
    test_lib::run_tests();
    0
}
