#![no_std]
#![no_main]
#![allow(dead_code)]
extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{fence, AtomicBool, Ordering};
use core::time::Duration;

use dlmalloc::Dlmalloc;

#[global_allocator]
static ALLOCATOR: GlobalDlmalloc = GlobalDlmalloc;

struct GlobalDlmalloc;

static mut DLMALLOC: Dlmalloc = Dlmalloc::new();

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::lock();
        let ptr = DLMALLOC.malloc(layout.size(), layout.align());
        Self::unlock();
        ptr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        Self::lock();
        DLMALLOC.free(ptr, layout.size(), layout.align());
        Self::unlock();
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::lock();
        let ptr = DLMALLOC.calloc(layout.size(), layout.align());
        Self::unlock();
        ptr
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        Self::lock();
        let ptr = DLMALLOC.realloc(ptr, layout.size(), layout.align(), new_size);
        Self::unlock();
        ptr
    }
}

static LOCK: AtomicBool = AtomicBool::new(false);
impl GlobalDlmalloc {
    fn lock() {
        while LOCK
            .compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {}
        fence(Ordering::SeqCst);
    }

    fn unlock() {
        LOCK.store(false, Ordering::SeqCst);
    }
}

macro_rules! run_test {
    ($func: expr) => {{
        unix_print::unix_print!("Running test {} ... ", stringify!($func));
        let __start = tiny_std::time::MonotonicInstant::now();
        $func();
        let __elapsed = __start.elapsed().as_secs_f32();
        unix_print::unix_println!("[OK] - {:.3} seconds", __elapsed);
    }};
}

#[no_mangle]
pub fn main() -> i32 {
    unix_print::unix_eprintln!("Starting alloc main");
    run_test!(test_read_env);
    run_test!(test_cmd_no_args);
    run_test!(thread_panics);
    run_test!(thread_parallel_compute);
    run_test!(thread_shared_memory_sequential_count);
    run_test!(thread_shared_memory_parallel_count);
    run_test!(thread_shared_memory_parallel_count_no_join);
    run_test!(thread_shared_memory_parallel_count_panics);
    run_test!(thread_shared_memory_parallel_count_panics_no_join);
    0
}

fn test_read_env() {
    let v = tiny_std::env::var("HOME").unwrap();
    assert_eq!("/home/gramar", v);
}

#[allow(dead_code)]
fn test_cmd_no_args() {
    let mut chld = tiny_std::process::Command::new("/usr/bin/uname")
        .unwrap()
        .spawn()
        .unwrap();
    chld.wait().unwrap();
}

fn thread_panics() {
    let t = tiny_std::thread::spawn(|| panic!("This is expected!")).unwrap();
    assert!(t.join().is_none());
}

const THREAD_LOOP_COUNT: usize = 1_000;

fn thread_parallel_compute() {
    let run_for = THREAD_LOOP_COUNT;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let handle = tiny_std::thread::spawn(move || {
            tiny_std::thread::sleep(Duration::from_millis(2)).unwrap();
            15
        })
        .unwrap();
        handles.push(handle);
    }
    let mut sum = 0;
    for handle in handles {
        sum += handle.join().unwrap();
    }
    assert_eq!(run_for * 15, sum);
}

fn thread_shared_memory_sequential_count() {
    let run_for = THREAD_LOOP_COUNT;
    let mut sum = 0;
    for _ in 0..run_for {
        let handle = tiny_std::thread::spawn(move || 1).unwrap();
        sum += handle.join().unwrap();
    }
    assert_eq!(run_for, sum);
}

fn thread_shared_memory_parallel_count() {
    let cnt = alloc::sync::Arc::new(tiny_std::rwlock::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
        })
        .unwrap();
        handles.push(handle);
    }
    for handle in handles {
        assert!(handle.join().is_some());
    }
    assert_eq!(run_for, *cnt.read());
}

fn thread_shared_memory_parallel_count_no_join() {
    let cnt = alloc::sync::Arc::new(tiny_std::rwlock::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let _handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
        })
        .unwrap();
    }
    while *cnt.read() < run_for {}
}

fn thread_shared_memory_parallel_count_panics() {
    let cnt = alloc::sync::Arc::new(tiny_std::rwlock::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
            panic!("Die")
        })
        .unwrap();
        handles.push(handle);
    }
    for handle in handles {
        assert!(handle.join().is_none());
    }
    assert_eq!(run_for, *cnt.read());
}

fn thread_shared_memory_parallel_count_panics_no_join() {
    let cnt = alloc::sync::Arc::new(tiny_std::rwlock::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let _handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
            panic!("Die")
        })
        .unwrap();
    }
    while run_for > *cnt.read() {}
    assert_eq!(run_for, *cnt.read());
}
