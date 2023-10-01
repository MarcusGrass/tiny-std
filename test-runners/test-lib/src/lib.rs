#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod with_alloc;

use core::time::Duration;
#[cfg(feature = "alloc")]
use with_alloc::{spawn_no_args, spawn_with_args};

#[cfg(not(feature = "alloc"))]
mod no_alloc;

#[cfg(feature = "threaded")]
mod threaded;

#[cfg(not(feature = "alloc"))]
use no_alloc::{spawn_no_args, spawn_with_args};

use tiny_std::UnixStr;

#[macro_export]
macro_rules! run_test {
    ($func: expr) => {{
        tiny_std::print!("Running test {} ... ", stringify!($func));
        let __start = tiny_std::time::MonotonicInstant::now();
        $func();
        let __elapsed = __start.elapsed().as_secs_f32();
        tiny_std::println!("[OK] - {:.3} seconds", __elapsed);
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
    tiny_std::println!("Running minimal feature set tests");
    run_test!(get_env);
    run_test!(get_args);
    run_test!(spawn_no_args);
    run_test!(spawn_with_args);
    run_test!(get_aux_values);
    run_test!(get_time);
}

fn get_env() {
    let v_unix = tiny_std::env::var_unix(UnixStr::try_from_str("HOME\0").unwrap()).unwrap();
    let v = tiny_std::env::var("HOME").unwrap();
    assert_eq!(v, v_unix);
    if is_ci() {
        assert_eq!("/home/runner", v);
    } else {
        assert_eq!("/home/gramar", v);
    }
}

fn get_args() {
    let mut args = tiny_std::env::args();
    args.next();
    let arg = args.next().unwrap().unwrap();
    assert_eq!("dummy_arg", arg);
    let mut os_args = tiny_std::env::args_os();
    os_args.next();
    let os_arg = os_args.next().unwrap();
    assert_eq!("dummy_arg", os_arg.as_str().unwrap());
}

fn get_aux_values() {
    // 16 random bytes
    let random = tiny_std::elf::aux::get_random();
    assert_ne!(0u128, random.unwrap());
    let uid = tiny_std::elf::aux::get_uid();
    let gid = tiny_std::elf::aux::get_gid();
    if is_ci() {
        assert!(uid > 0, "Expected uid to be above 0 on CI, got {}", uid);
        assert!(gid > 0, "Expected gid to be above 0 on CI, got {}", gid);
    } else {
        assert_eq!(1000, uid);
        assert_eq!(1000, gid);
    }
}

fn get_time() {
    let now = tiny_std::time::Instant::now();
    tiny_std::thread::sleep(Duration::from_micros(10)).unwrap();
    let later = tiny_std::time::Instant::now();
    assert!(later > now, "Expected {later:?} to be after {now:?}");
    let now = tiny_std::time::SystemTime::now();
    tiny_std::thread::sleep(Duration::from_micros(10)).unwrap();
    let later = tiny_std::time::SystemTime::now();
    assert!(later > now, "Expected {later:?} to be after {now:?}");
}

fn is_ci() -> bool {
    !matches!(
        tiny_std::env::var_unix(UnixStr::try_from_str("CI\0").unwrap()),
        Err(tiny_std::env::VarError::Missing)
    )
}
