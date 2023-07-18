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
#[allow(
    clippy::used_underscore_binding,
    clippy::cast_ptr_alignment,
    clippy::cast_possible_truncation
)]
unsafe extern "C" fn __proxy_main(stack_ptr: *const u8, _dynv: *const usize) {
    // Fist 8 bytes is a u64 with the number of arguments
    let argc = *stack_ptr.cast::<u64>();
    // Directly followed by those arguments, bump pointer by 8
    let argv = stack_ptr.add(8).cast::<*const u8>();
    let ptr_size = core::mem::size_of::<usize>();
    // Directly followed by a pointer to the environment variables, it's just a null terminated string.
    // This isn't specified in Posix and is not great for portability, but we're targeting Linux so it's fine
    let env_offset = 8 + argc as usize * ptr_size + ptr_size;
    // Bump pointer by combined offset
    let envp = stack_ptr.add(env_offset).cast::<*const u8>();
    #[cfg(feature = "aux")]
    {
        let mut null_offset = 0;
        loop {
            let val = *(envp.add(null_offset));
            if val as usize == 0 {
                break;
            }
            null_offset += 1;
        }
        let addr = envp.add(null_offset).cast::<usize>();
        let aux_v_ptr = addr.add(1);
        let aux = crate::elf::aux::AuxValues::from_auxv(aux_v_ptr);
        crate::elf::dynlink::relocate_symbols(_dynv, &aux);
        crate::elf::aux::AUX_VALUES = aux;
    }

    crate::env::ENV.arg_c = argc;
    crate::env::ENV.arg_v = argv;
    crate::env::ENV.env_p = envp;

    #[cfg(feature = "vdso")]
    {
        crate::elf::vdso::init_vdso_get_time();
    }
    #[cfg(feature = "alloc")]
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
