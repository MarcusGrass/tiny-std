use alloc::boxed::Box;
use core::arch::global_asm;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use rusl::platform::{CloneFlags, MapAdditionalFlags, MapRequiredFlag, MemoryProtection};
use rusl::unistd::mmap;

use crate::error::Result;
use crate::rwlock::futex_wait_fast;

pub struct JoinHandle<T> {
    inner: NonNull<ThreadSharedValue<T>>,
    futex: NonNull<AtomicU32>,
}

struct ThreadSharedValue<T> {
    needs_dealloc: AtomicBool,
    value: UnsafeCell<Option<T>>,
}

impl<T> ThreadSharedValue<T> {
    fn new() -> Self {
        Self {
            needs_dealloc: AtomicBool::new(false),
            value: UnsafeCell::new(None),
        }
    }
}

// Kernel will set this to 0 on child exit https://man7.org/linux/man-pages/man2/set_tid_address.2.html
const UNFINISHED: u32 = 1;

impl<T> JoinHandle<T> {
    /// If the thread has panicked, this will return `None`
    #[must_use]
    pub fn join(self) -> Option<T> {
        // The OS will change to futex value to 0 and then wake it when the thread finishes.
        unsafe {
            futex_wait_fast(self.futex.as_ref(), UNFINISHED);
        }
        // The thread has completed, we have exclusive access to the memory
        unsafe {
            drop(Box::from_raw(self.futex.as_ptr()));
            let val = self.inner.as_ptr().read().value.into_inner();
            // We have exclusive access so we don't need to run the destructor anymore
            // just dealloc and forget.
            drop(Box::from_raw(self.inner.as_ptr()));
            core::mem::forget(self);
            val
        }
    }
}

impl<T> Drop for JoinHandle<T> {
    // Todo: Try to discover if dropping without a `futex`-dealloc leaks memory.
    // We can't just dealloc it here because the kernel will write to the futex address later,
    // and that's a use-after-free, it's pretty hard to find since it's 32-bits.
    fn drop(&mut self) {
        unsafe {
            // We signal to the thread that it needs to dealloc this shared variable.
            // If it already needs dealloc we dealloc here
            if self
                .inner
                .as_ref()
                .needs_dealloc
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                drop(Box::from_raw(self.inner.as_ptr()));
            }
        }
    }
}

/// Spawn a thread that will run the provided function
/// # Errors
/// Failure to mmap the thread's stack.
pub fn spawn<T, F: FnOnce() -> T>(func: F) -> Result<JoinHandle<T>>
where
    F: Send + 'static,
    T: Send + 'static,
{
    let flags = CloneFlags::CLONE_VM
        | CloneFlags::CLONE_FS
        | CloneFlags::CLONE_FILES
        | CloneFlags::CLONE_SIGHAND
        | CloneFlags::CLONE_THREAD
        | CloneFlags::CLONE_SYSVSEM
        | CloneFlags::CLONE_CHILD_CLEARTID
        | CloneFlags::CLONE_DETACHED;
    let stack_sz = 8192 * 16 * 16;
    let guard_sz = 0;
    let size = guard_sz + stack_sz;

    // Create a NonNull pointer to the data from our box.
    let payload: NonNull<ThreadSharedValue<T>> =
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(ThreadSharedValue::new()))) };
    let pl_c = payload;
    let df_ptr: Box<dyn FnOnce()> = Box::new(move || {
        unsafe {
            let func_ret = func();
            let _ = (*pl_c.as_ref().value.get()).insert(func_ret);
            // Signal that this thread is done with the value and it can be safely
            // deallocated.
            // If it fails, it means the caller has dropped the JoinHandle, then we need to dealloc here.
            if pl_c
                .as_ref()
                .needs_dealloc
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                drop(Box::from_raw(pl_c.as_ptr()));
            }
        }
    });
    // We need to double box here because
    // 1. We need to access through a box, because we can't cast into a *mut dyn FnOnce(), because
    // fat pointer.
    // 2. We can't refer to the box we create by address on the stack, because we will risk accessing
    // it after this part of the stack is destroyed/overwritten/whatever.
    let raw_df = Box::into_raw(Box::new(df_ptr));

    let map_ptr = unsafe {
        mmap(
            None,
            NonZeroUsize::new_unchecked(size),
            MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
            MapRequiredFlag::MapPrivate,
            MapAdditionalFlags::MAP_ANONYMOUS,
            None,
            0,
        )?
    };
    // Stack grows downward
    let mut stack = map_ptr + size;
    // shift down a bit, unsure exactly why, doesn't really matter if we do or don't actually
    stack -= stack % core::mem::size_of::<usize>();
    stack -= core::mem::size_of::<StartArgs>();
    let args = stack as *mut StartArgs;
    unsafe {
        (*args).start_func = thread_go as _;
        (*args).start_arg = raw_df as usize;
    }

    let die_futex =
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(AtomicU32::new(UNFINISHED)))) };
    #[allow(clippy::cast_possible_truncation)]
    unsafe {
        __clone(
            start_fn as _,
            stack,
            flags.bits() as i32,
            args as _,
            0,
            die_futex.as_ptr() as usize,
            map_ptr,
            stack_sz,
        );
    }
    Ok(JoinHandle {
        inner: payload,
        futex: die_futex,
    })
}

#[repr(C)]
struct StartArgs {
    start_func: usize,
    start_arg: usize,
}

unsafe extern "C" fn start_fn(ptr: *mut StartArgs) -> i32 {
    let args = ptr.read();
    let func = args.start_func as *const ();
    let run = core::mem::transmute::<*const (), fn(*const c_void) -> i32>(func);
    (run)(args.start_arg as _)
}

/// We need a c-style fn ptr here
extern "C" fn thread_go(ptr: *const c_void) -> i32 {
    unsafe {
        // We need a rust-style fn ptr here
        let bb_fn: Box<Box<dyn FnOnce()>> = Box::from_raw(ptr as *mut Box<dyn FnOnce()>);
        (bb_fn)();
    }
    0
}

extern "C" {
    /// Largely a ripoff of `musl`'s clone wrap with some extra steps.
    /// The function does a bit of register shuffling, calls the clone syscall,
    /// executes the provided function with its args, unmaps the stack pointer,
    /// then exits.
    ///
    /// Syscall conventions are on 5 args:
    /// - nr -> x86: `rax`, aarch64: `x8`
    /// - a1 -> x86 `rdi`, aarch64: `x0`
    /// - a2 -> x86: `rsi`, aarch64: `x1`
    /// - a3 -> x86: `rdx`, aarch64: `x2`
    /// - a4 -> x86: `r10`, aarch64: `x3`
    /// - a5 -> x86: `r8`, aarch64: `x4`
    ///
    /// Clone syscall expected args:
    /// - `flags`, x86: arg1/`rdi`, aarch64: arg1/`x0`
    /// - `stack_ptr`, x86: arg2/`rsi`, aarch64: arg2/`x1`
    /// - `parent_tid`, x86: arg3/`rdx`, aarch64: arg3/`x2`
    /// - `child_tid`, x86: arg4/`r10`, aarch64: arg5/`x4`
    /// - `tls`, x86: arg5/`r8`, aarch64: arg4/`x3`
    ///
    /// Input arguments with c-conventions:
    /// start_fn: `rdi` / `x0`,
    /// stack_ptr: `rsi` / `x1`,
    /// flags: `rdx` / `x2`,
    /// args_ptr: `rcx` / `x3`,
    /// pt_ptr: `r8` / `x4`,
    /// tl_lk_ptr: `r9` / `x5`,
    /// stack_unmap_ptr: `stack first 8 bytes` / `x6`,
    /// stack_sz: `stack second 8 bytes` / `x7`,
    fn __clone(
        start_fn: usize,
        stack_ptr: usize,
        flags: i32,
        args_ptr: usize,
        pt_ptr: usize,
        tl_lk_ptr: usize,
        stack_unmap_ptr: usize,
        stack_sz: usize,
    ) -> i32;
}

#[cfg(target_arch = "x86_64")]
global_asm!(
    ".text",
    ".global __clone",
    ".hidden __clone",
    ".type   __clone,@function",
    "__clone:",
    // Zero syscall nr register ax (eax = 32bit ax)
    "xor eax, eax",
    // Move 56 into the lower 8 bits of ax (al = 8bit ax)
    "mov al, 56",
    // Move start function into r11, scratch register
    "mov r11, rdi",
    // Move flags into rdi, syscall arg 1 register
    "mov rdi, rdx",
    // Move parent_tid_ptr into rdx, syscall arg 3 register (not using, zero it instead)
    "xor rdx, rdx",
    // pt_ptr already in r8, syscall arg 5 register
    // Move the first argument that went on the stack into syscall arg 4 register (our arg 6)
    "mov r10, r9",
    // Move start function into r9
    "mov r9, r11",
    // Align stack ptr to -16
    "and rsi, -16",
    // Move down 8 bytes on the stack ptr
    "sub rsi,8",
    // Move args onto the the top of the stack
    "mov [rsi], rcx",
    // Move down 8 bytes more on the stack ptr
    "sub rsi, 8",
    // Move next arg that went on stack into rcx
    "mov rcx, [8 + rsp]",
    // Move next arg onto our new stack
    "mov [rsi], rcx",
    // Move next arg into rcx
    "mov rcx, [16 + rsp]",
    // Move down stack ptr
    "sub rsi, 8",
    // Move the last arg onto the new stack
    "mov [rsi], rcx",
    // Make clone syscall
    "syscall",
    // Check if the syscall return vaulue is 0
    "test eax, eax",
    // if not zero, return (we're the calling thread)
    "jnz 1f",
    // Zero the base pointer
    "xor ebp, ebp",
    // Pop the stack len off the provided stack into callee saved
    "pop r13",
    // Pop the stack_ptr off the provided stack into callee saved
    "pop r12",
    // Pop the start fn args off the provided stack
    "pop rdi",
    // Call the function we saved in r9, rdi first arg
    "call r9",
    // move the return value into callee saved register
    "mov r14, rax",
    // Zero eax
    "xor eax, eax",
    // Move MUNMAP syscall into ax
    "mov al, 11",
    // Stack ptr as the first call
    "mov rdi, r12",
    // Stack len as the second call
    "mov rsi, r13",
    // Call function, unmapping the stack
    "syscall",
    // Clear the output register, we can't use the return value anyway
    "xor eax,eax",
    // Move EXIT syscall into ax
    "mov al, 60",
    // Move return value into syscall arg 1 register
    "mov rdi, r14",
    // Make exit syscall
    "syscall",
    // Halt thread
    "hlt",
    "1: ret",
);

#[cfg(target_arch = "aarch64")]
global_asm!(
    ".global __clone",
    ".hidden __clone",
    ".type   __clone,@function",
    "__clone:",
    // Align stack
    "and x1, x1, #-16",
    // Store Pair of Registers, Pre-index-mode: push function and args onto stack, decrement by 16 bits (move stack pointer)
    "stp x0, x3, [x1, #-16]!",
    // Store Pair of Registers, Pre-index-mode: push stack_unmap_ptr and stack_sz onto stack, decrement by 16 bits (move stack pointer)
    "stp x7, x6, [x1, #-16]!",
    // syscall
    // Move word into double-word arg 1 syscall register
    "uxtw x0, w2",
    // Zero ptid, not using, arg 3 syscall register
    "eor x2, x2, x2",
    // Move pt_ptr into arg 4 syscall register
    "mov x3, x4",
    // Move child tid ptr into arg 5 syscall register
    "mov x4, x5",
    "mov x8, #220",
    // Make clone syscall
    "svc #0",
    // If 0, branch, clone returns 0 on the child thread
    "cbz x0, 1f",
    // Return on parent
    "ret",
    // Load Pair of Registers, Post-index-mode: pull out two double-words (16 bytes total) out of the stack pointer, increment pointer by 16.
    // Stack size goes in callee saved register `x20`, stack size goes in `x21`
    "1:	ldp x21, x20, [sp], #16",
    // Load Pair of Registers, Post-index-mode: pull out two double-words (16 bytes total) out of the stack pointer, increment pointer by 16.
    // Function goes in `x1` (doesn't really matter where), args goes in `x0` (fn first arg)
    "ldp x1, x0, [sp], #16",
    // Branch link register (essentially call the function) our args are in `x0`
    "blr x1",
    // Move fn output into x19
    "mov x19, x0",
    // Move stack_ptr into syscall arg 1 register
    "mov x0, x20",
    // Move stack_sz into syscall arg 2 register
    "mov x1, x21",
    "mov x8, #215",
    // Make munmap syscall, unmap stack
    "svc #0",
    // Move fn result into syscall arg 1 register
    "mov x0, x19",
    "mov x8, #93",
    // Make exit syscall, thread is complete
    "svc #0",
);
