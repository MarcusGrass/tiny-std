pub struct Env {
    pub arg_c: u64,
    pub arg_v: *const *const u8,
    pub env_p: *const *const u8,
}

/// Resolve environment, aux values, does not relocate symbols.
/// Do not use if symbol relocation is wanted or required, such as when compiling a static-pie-linked
/// executable, this will segfault.
/// # Safety
/// Only safe if provided with the Os's `stack_ptr` and `_dynv`, without any modification
#[must_use]
#[inline(always)]
#[cfg(not(feature = "aux"))]
#[allow(
    clippy::cast_ptr_alignment,
    clippy::cast_possible_truncation,
    clippy::used_underscore_binding
)]
pub unsafe fn resolve(stack_ptr: *const u8, _dynv: *const usize) -> Env {
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
    Env {
        arg_c: argc,
        arg_v: argv,
        env_p: envp,
    }
}

/// Resolve environment, aux values, and relocate symbols
/// # Safety
/// Only safe if provided with the Os's `stack_ptr` and `_dynv`, without any modification
#[must_use]
#[inline(always)]
#[cfg(feature = "aux")]
#[allow(
    clippy::cast_ptr_alignment,
    clippy::cast_possible_truncation,
    clippy::used_underscore_binding
)]
pub unsafe fn resolve(
    stack_ptr: *const u8,
    _dynv: *const usize,
) -> (Env, crate::elf::aux::AuxValues) {
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
    (
        Env {
            arg_c: argc,
            arg_v: argv,
            env_p: envp,
        },
        aux,
    )
}
