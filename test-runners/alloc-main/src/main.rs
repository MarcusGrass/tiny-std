#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, fence, Ordering};

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
        while LOCK.compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {

        }
        fence(Ordering::SeqCst);
    }

    fn unlock() {
        LOCK.store(false, Ordering::SeqCst);
    }
}

/// Panic handler
#[panic_handler]
pub fn on_panic(info: &core::panic::PanicInfo) -> ! {
    unix_print::unix_eprintln!("Panicked {info}");
    tiny_std::process::exit(1)
}

#[no_mangle]
pub fn main() -> i32 {
    unix_print::unix_eprintln!("Starting");
    let v = tiny_std::env::var("HOME").unwrap();
    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!("/home/gramar", v);
    }
    #[cfg(target_arch = "aarch64")]
    {
        // Running this in a container
        assert!(v == "/root" || v == "/");
    }
    test_cmd_no_args();


    thread_panics_ok();
    let cnt = alloc::sync::Arc::new(tiny_std::rwlock::RwLock::new(0));
    let run_for = 1_000;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let handle = tiny_std::thread::spawn(move || {
            tiny_std::thread::sleep(core::time::Duration::from_secs_f32(0.25)).unwrap();
            *cnt_c.write() += 1;
        }).unwrap();
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    while *cnt.read() != run_for {}
    unix_print::unix_println!("Val = {}", *cnt.read());
    0
}

fn test_cmd_no_args() {
    unix_print::unix_eprintln!("Spwaning no args");
    let mut chld = tiny_std::process::Command::new("/usr/bin/uname")
        .unwrap()
        .spawn()
        .unwrap();
    chld.wait().unwrap();
}

fn thread_panics_ok() {
    let t = tiny_std::thread::spawn(|| panic!("This is expected!")).unwrap();
    assert!(t.join().is_none());
}