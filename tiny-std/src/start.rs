//! Support for starting a `Rust` program without libc dependencies
//! for the time being it requires the nightly feature `naked_function` to start
//! a `Rust` application this way.

#[cfg(all(feature = "symbols", feature = "start"))]
use crate::process::exit;

/// We have to mimic libc globals here sadly, we rip the environment off the first pointer of the stack
/// in the start method. Should never be modified ever, just set on start
pub(crate) static mut ENV: Env = Env {
    arg_c: 0,
    arg_v: core::ptr::null(),
    env_p: core::ptr::null(),
};

pub(crate) struct Env {
    pub(crate) arg_c: u64,
    pub(crate) arg_v: *const *const u8,
    pub(crate) env_p: *const *const u8,
}

#[cfg(feature = "aux")]
pub(crate) struct AuxV {
    ptr: *const usize,
    locations: [usize; rusl::platform::AUX_CNT / 2 + 1],
}

/// A vector of dynamic values supplied by the OS
#[cfg(feature = "aux")]
pub(crate) static mut AUX_V: AuxV = AuxV {
    ptr: core::ptr::null(),
    locations: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
};

/// VDSO dynamically provided function pointer to CLOCK_GET_TIME
#[cfg(feature = "vdso")]
pub(crate) static mut VDSO_CLOCK_GET_TIME: Option<
    extern "C" fn(i32, *mut rusl::platform::TimeSpec) -> i32,
> = None;

/// Attempts to find the specified aux value from the OS supplied aux vector
#[cfg(feature = "aux")]
pub fn get_aux_value(val: rusl::platform::AuxValue) -> Option<usize> {
    unsafe {
        for i in 0..AUX_V.locations.len() {
            if AUX_V.locations[i] == val.bits() {
                return Some(AUX_V.ptr.add(2 * i + 1).read());
            }
        }
    }
    None
}

/// Skip lang item feature which will never stabilize and just put the symbol in
/// # Safety
/// Just a symbol that's necessary
#[no_mangle]
#[cfg(all(feature = "symbols", feature = "start"))]
pub unsafe fn rust_eh_personality() {}

// Binary entrypoint
#[cfg(all(feature = "symbols", feature = "start", target_arch = "x86_64"))]
core::arch::global_asm!(
    ".text",
    ".global _start",
    ".type _start,@function",
    "_start:",
    "mov rdi, rsp",
    "call __proxy_main"
);

#[cfg(all(feature = "symbols", feature = "start", target_arch = "aarch64"))]
core::arch::global_asm!(
    ".text",
    ".global _start",
    ".type _start,@function",
    "_start:",
    "MOV X0, sp",
    "bl __proxy_main"
);

/// Called with a pointer to the top of the stack
#[no_mangle]
#[cfg(all(feature = "symbols", feature = "start"))]
unsafe fn __proxy_main(stack_ptr: *const u8) {
    // Fist 8 bytes is a u64 with the number of arguments
    let argc = *(stack_ptr as *const u64);
    // Directly followed by those arguments, bump pointer by 8
    let argv = stack_ptr.add(8) as *const *const u8;
    let ptr_size = core::mem::size_of::<usize>();
    // Directly followed by a pointer to the environment variables, it's just a null terminated string.
    // This isn't specified in Posix and is not great for portability, but we're targeting Linux so it's fine
    let env_offset = 8 + argc as usize * ptr_size + ptr_size;
    // Bump pointer by combined offset
    let envp = stack_ptr.add(env_offset) as *const *const u8;
    unsafe {
        ENV.arg_c = argc;
        ENV.arg_v = argv;
        ENV.env_p = envp;
    }
    let mut null_offset = 0;
    loop {
        if envp.add(null_offset).read().is_null() {
            break;
        }
        null_offset += 1;
    }
    #[cfg(feature = "aux")]
    {
        let aux_v = envp.add(null_offset + 1) as *const usize;
        collect_aux_values(aux_v);
    }
    #[cfg(feature = "vdso")]
    {
        if let Some(elf_start) = get_aux_value(rusl::platform::AuxValue::AT_SYSINFO_EHDR) {
            let get_time = crate::vdso::find_vdso_clock_get_time(elf_start as _);
            VDSO_CLOCK_GET_TIME = get_time;
        }
    }
    #[cfg(feature = "alloc")]
    {
        // To be able to get the thread tls in the panic handler we need to set up
        // thread local storage for the main thread. But we shouldn't dealloc this thread's stack
        // on a panic so set stack_info to `None`.
        // No spooky pointers into the stack here, this will live for the lifetime of
        // the thread so we'll just alloc and leak it.
        let mut main_thread_tls = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(
            crate::thread::spawn::ThreadLocalStorage {
                self_addr: 0,
                stack_info: None,
            },
        ));
        let self_addr = main_thread_tls as usize;
        (*main_thread_tls).self_addr = self_addr;
        // x86_64 ARCH_GET_FS
        #[cfg(target_arch = "x86_64")]
        {
            sc::syscall!(ARCH_PRCTL, 0x1002, self_addr);
        }
        #[cfg(target_arch = "aarch64")]
        {
            core::arch::asm!("msr tpidr_el0, {x}", x = in(reg) main_thread_tls);
        }
    }
    let code = main();
    exit(code);
}

#[inline]
#[cfg(feature = "aux")]
unsafe fn collect_aux_values(aux_v: *const usize) {
    AUX_V.ptr = aux_v;
    let mut num_aux_values = 0;
    loop {
        let key = aux_v.add(num_aux_values).read();
        if key == 0 {
            break;
        }
        AUX_V.locations[num_aux_values / 2] = key;
        num_aux_values += 2;
    }
}

#[cfg(all(feature = "symbols", feature = "start"))]
extern "Rust" {
    // The user's main
    fn main() -> i32;
}
