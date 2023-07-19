#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod with_alloc;

#[cfg(feature = "alloc")]
use with_alloc::{spawn_no_args, spawn_with_args};

#[cfg(not(feature = "alloc"))]
mod no_alloc;

#[cfg(feature = "threaded")]
mod threaded;

#[cfg(not(feature = "alloc"))]
use no_alloc::{spawn_no_args, spawn_with_args};

#[macro_export]
macro_rules! run_test {
    ($func: expr) => {{
        unix_print::unix_print!("Running test {} ... ", stringify!($func));
        let __start = tiny_std::time::MonotonicInstant::now();
        $func();
        let __elapsed = __start.elapsed().as_secs_f32();
        unix_print::unix_println!("[OK] - {:.3} seconds", __elapsed);
    }};
}

pub fn run_tests() {
    run_minimal_feature_set();
    #[cfg(feature = "threaded")]
    {
        threaded::run_threaded_tests();
    }
}

fn run_minimal_feature_set() {
    unix_print::unix_println!("Running minimal feature set tests");
    run_test!(get_env);
    run_test!(get_args);
    run_test!(spawn_no_args);
    run_test!(spawn_with_args);
    run_test!(get_aux_values);
    run_test!(get_time);
}

fn get_env() {
    let v = tiny_std::env::var("HOME").unwrap();
    assert_eq!("/home/gramar", v);
}

fn get_args() {
    let mut args = tiny_std::env::args();
    args.next();
    let arg = args.next().unwrap().unwrap();
    assert_eq!("dummy_arg", arg);
    let mut os_args = tiny_std::env::args_os();
    let os_arg = os_args.next().unwrap();
    assert_eq!("dummy_arg", os_arg.as_str().unwrap());
}

fn get_aux_values() {
    // 16 random bytes
    let random = tiny_std::elf::aux::get_random();
    assert_ne!(0u128, random.unwrap());
    let uid = tiny_std::elf::aux::get_uid();
    let gid = tiny_std::elf::aux::get_gid();
    assert_eq!(1000, uid);
    assert_eq!(1000, gid);
}

fn get_time() {
    let now = tiny_std::time::Instant::now();
    let later = tiny_std::time::Instant::now();
    assert!(later > now);
    let now = tiny_std::time::SystemTime::now();
    let later = tiny_std::time::SystemTime::now();
    assert!(later > now);
}
