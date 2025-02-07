//! Support for starting a `Rust` program without libc dependencies

// Binary entrypoint
#[cfg(all(feature = "start", feature = "symbols", target_arch = "x86_64"))]
core::arch::global_asm!(
    ".text",
    ".global _start",
    ".type _start,@function",
    "_start:",
    "xor rbp,rbp",
    "mov rdi, rsp",
    ".weak _DYNAMIC",
    ".hidden _DYNAMIC",
    "lea rsi, [rip + _DYNAMIC]",
    "and rsp,-16",
    "call __proxy_main"
);

#[cfg(all(feature = "start", feature = "symbols", target_arch = "aarch64"))]
core::arch::global_asm!(
    ".text",
    ".global _start",
    ".type _start,@function",
    "_start:",
    "mov x29, #0",
    "mov x30, #0",
    "mov x0, sp",
    ".weak _DYNAMIC",
    ".hidden _DYNAMIC",
    "adrp x1, _DYNAMIC",
    "add x1, x1, #:lo12:_DYNAMIC",
    "and sp, x0, #-16",
    "bl __proxy_main"
);

/// Called with a pointer to the top of the stack
#[no_mangle]
#[cfg(feature = "symbols")]
#[expect(clippy::used_underscore_binding)]
unsafe extern "C" fn __proxy_main(stack_ptr: *const u8, _dynv: *const usize) {
    #[cfg(feature = "aux")]
    {
        let (env, aux) = tiny_start::start::resolve(stack_ptr, _dynv);
        crate::env::ENV.arg_c = env.arg_c;
        crate::env::ENV.arg_v = env.arg_v;
        crate::env::ENV.env_p = env.env_p;
        crate::elf::aux::AUX_VALUES = aux;
    }
    #[cfg(not(feature = "aux"))]
    {
        let env = tiny_start::start::resolve(stack_ptr, _dynv);
        crate::env::ENV.arg_c = env.arg_c;
        crate::env::ENV.arg_v = env.arg_v;
        crate::env::ENV.env_p = env.env_p;
    }

    #[cfg(feature = "vdso")]
    {
        crate::elf::vdso::init_vdso_get_time();
    }

    #[cfg(feature = "threaded")]
    {
        // To be able to get the thread tls in the panic handler we need to set up
        // thread local storage for the main thread. But we shouldn't dealloc this thread's stack
        // on a panic so set stack_info to `None`.
        // No spooky pointers into the stack here, this will live for the lifetime of
        // the thread so we'll just alloc and leak it.
        let main_thread_tls = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(
            crate::thread::spawn::ThreadLocalStorage {
                self_addr: 0,
                stack_info: None,
            },
        ));
        let self_addr = main_thread_tls as usize;
        (*main_thread_tls).self_addr = self_addr;
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 ARCH_GET_FS
            sc::syscall!(ARCH_PRCTL, 0x1002, self_addr);
        }
        #[cfg(target_arch = "aarch64")]
        {
            core::arch::asm!("msr tpidr_el0, {x}", x = in(reg) main_thread_tls);
        }
    }
    let code = main();
    crate::process::exit(code);
}

#[cfg(feature = "symbols")]
extern "Rust" {
    // The user's main
    fn main() -> i32;
}
