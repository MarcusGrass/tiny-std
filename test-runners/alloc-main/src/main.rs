#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

use dlmalloc::Dlmalloc;

#[global_allocator]
static ALLOCATOR: SingleThreadedAlloc = SingleThreadedAlloc::new();

struct SingleThreadedAlloc {
    inner: UnsafeCell<Dlmalloc>,
}

impl SingleThreadedAlloc {
    pub(crate) const fn new() -> Self {
        SingleThreadedAlloc {
            inner: UnsafeCell::new(Dlmalloc::new()),
        }
    }
}

unsafe impl GlobalAlloc for SingleThreadedAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self.inner.get()).malloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self.inner.get()).free(ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (*self.inner.get()).calloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        (*self.inner.get()).realloc(ptr, layout.size(), layout.align(), new_size)
    }
}

/// Extremely unsafe, this program is not thread safe at all will almost immediately segfault on more threads
unsafe impl Sync for SingleThreadedAlloc {}

unsafe impl Send for SingleThreadedAlloc {}

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
        assert_eq!("/root", v);
    }
    test_cmd_no_args();
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
