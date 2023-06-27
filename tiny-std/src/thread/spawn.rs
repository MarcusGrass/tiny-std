use alloc::alloc::dealloc;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::arch::global_asm;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::fmt::Debug;
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use sc::nr::MUNMAP;

use rusl::platform::{CloneFlags, MapAdditionalFlags, MapRequiredFlag, MemoryProtection};
use rusl::unistd::mmap;

use crate::error::Result;
use crate::rwlock::futex_wait_fast;

pub struct JoinHandle<T: Sized> {
    needs_dealloc: NonNull<AtomicBool>,
    value: NonNull<UnsafeCell<Option<T>>>,
    futex: NonNull<AtomicU32>,
}

// Kernel will set this to 0 on child exit https://man7.org/linux/man-pages/man2/set_tid_address.2.html
const UNFINISHED: u32 = 1;

impl<T: Sized> JoinHandle<T> {
    /// If the thread has panicked, this will return `None`
    #[must_use]
    pub fn join(self) -> Option<T> {
        // The OS will change to futex value to 0 and then wake it when the thread finishes.
        unsafe {
            futex_wait_fast(self.futex.as_ref(), UNFINISHED);
            // The thread has completed, we have exclusive access to the memory.
            // Pack it into a box, then consume the box to get the value off the heap.
            let val = Box::from_raw(self.value.as_ptr()).into_inner();
            // We have exclusive access so we don't need to run the destructor anymore
            // just dealloc and forget.
            dealloc(self.futex.as_ptr().cast(), Layout::new::<AtomicU32>());
            dealloc(
                self.needs_dealloc.as_ptr().cast(),
                Layout::new::<AtomicBool>(),
            );
            core::mem::forget(self);
            val
        }
    }
}

impl<T: Sized> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        unsafe {
            // We signal to the thread that it needs to dealloc this shared variable.
            // If it's already done, we're responsible for the cleanup.
            if self
                .needs_dealloc
                .as_ref()
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                // The thread got its work done first, we need to wait for it to exit, signalled
                // by the OS through the futex, then we know we have exclusive access to the memory.
                futex_wait_fast(self.futex.as_ref(), UNFINISHED);
                dealloc(
                    self.value.as_ptr().cast(),
                    Layout::new::<UnsafeCell<Option<T>>>(),
                );
                dealloc(self.futex.as_ptr().cast(), Layout::new::<AtomicU32>());
                dealloc(
                    self.needs_dealloc.as_ptr().cast(),
                    Layout::new::<AtomicBool>(),
                );
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct ThreadLocalStorage {
    // First arg needs to be a pointer to this struct, it's immediately dereferenced
    pub(crate) self_addr: usize,
    // Info on spawned threads that allow us to unmap the stack later
    pub(crate) stack_info: Option<ThreadDealloc>,
}

impl ThreadLocalStorage {
    #[inline]
    fn thread_stack_info(&self) -> &Option<ThreadDealloc> {
        #[cfg(target_arch = "x86_64")]
        {
            &self.stack_info
        }
        // On aarch64 we don't put anything onto the tls for the main thread,
        // we get the value out of the tls-register, if that register holds a zero double-word
        // then we have no storage and return None.
        #[cfg(target_arch = "aarch64")]
        {
            if self.self_addr == 0 {
                &None
            } else {
                &self.stack_info
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct ThreadDealloc {
    stack_addr: usize,
    stack_sz: usize,
    payload_ptr: usize,
    payload_layout: Layout,
    futex_ptr: usize,
    sync_ptr: usize,
}

/// Spawn a thread that will run the provided function
/// # Errors
/// Failure to mmap the thread's stack.
pub fn spawn<T: Debug + Copy, F: FnOnce() -> T>(func: F) -> Result<JoinHandle<T>>
where
    F: Send + 'static,
    T: Sized + Send + 'static,
{
    let flags = CloneFlags::CLONE_VM
        | CloneFlags::CLONE_FS
        | CloneFlags::CLONE_FILES
        | CloneFlags::CLONE_SIGHAND
        | CloneFlags::CLONE_THREAD
        | CloneFlags::CLONE_SYSVSEM
        | CloneFlags::CLONE_CHILD_CLEARTID
        | CloneFlags::CLONE_DETACHED
        | CloneFlags::CLONE_SETTLS;
    let stack_sz = 8192 * 16 * 16;
    let guard_sz = 0;
    let size = guard_sz + stack_sz;

    // Create a NonNull pointer to the data from our box.
    let payload: NonNull<UnsafeCell<Option<T>>> =
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(UnsafeCell::new(None)))) };
    let pl_c = payload;

    let futex_mem = Box::into_raw(Box::new(AtomicU32::new(UNFINISHED)));
    let needs_dealloc =
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(AtomicBool::new(false)))) };
    let die_futex = unsafe { NonNull::new_unchecked(futex_mem) };
    let payload_layout = unsafe {
        Layout::from_size_align_unchecked(
            core::mem::size_of::<UnsafeCell<Option<T>>>(),
            core::mem::align_of::<UnsafeCell<Option<T>>>(),
        )
    };
    let df_ptr: Box<dyn FnOnce()> = Box::new(move || {
        unsafe {
            // Run the function, if it panics, goto #[panic_handler].
            let func_ret = func();
            // The caller won't try to access the value until this thread exits.
            pl_c.as_ref().get().write(Some(func_ret));
            // Signal that this thread is done with the value and it can be safely
            // consumed.
            // If it fails, it means the caller has dropped the JoinHandle, then we need to dealloc here.
            if needs_dealloc
                .as_ref()
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                dealloc(pl_c.as_ptr().cast(), Layout::new::<UnsafeCell<Option<T>>>());
                dealloc(needs_dealloc.as_ptr().cast(), Layout::new::<AtomicBool>());
                // We need to set this thread's TID_ADDRESS ptr to null, or else
                // the kernel will try to update the value, and futex_wake on it, which will
                // cause a segfault.
                sc::syscall!(SET_TID_ADDRESS, 0);
                dealloc(die_futex.as_ptr().cast(), Layout::new::<AtomicU32>());
            }
            // Also dealloc the local storage for this thread, nobody needs that anymore
            dealloc(get_tls_ptr().cast(), Layout::new::<ThreadLocalStorage>());
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

    let mut tls = Box::into_raw(Box::new(ThreadLocalStorage {
        self_addr: 0,
        stack_info: Some(ThreadDealloc {
            stack_addr: map_ptr,
            stack_sz: size,
            payload_ptr: payload.as_ptr() as usize,
            payload_layout,
            futex_ptr: die_futex.as_ptr() as usize,
            sync_ptr: needs_dealloc.as_ptr() as usize,
        }),
    }));
    unsafe {
        (*tls).self_addr = tls as usize;
    }
    #[allow(clippy::cast_possible_truncation)]
    unsafe {
        __clone(
            start_fn as _,
            stack,
            flags.bits() as i32,
            args as _,
            tls as usize,
            die_futex.as_ptr() as usize,
            map_ptr,
            stack_sz,
        );
    }
    Ok(JoinHandle {
        needs_dealloc,
        value: payload,
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
    /// tls_ptr: `r8` / `x4`,
    /// child_tid_ptr: `r9` / `x5`,
    /// stack_unmap_ptr: `stack first 8 bytes` / `x6`,
    /// stack_sz: `stack second 8 bytes` / `x7`,
    fn __clone(
        start_fn: usize,
        stack_ptr: usize,
        flags: i32,
        args_ptr: usize,
        tls_ptr: usize,
        child_tid_ptr: usize,
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
    // Zero parent_tid_ptr from syscall arg 3 register (not using)
    "xor rdx, rdx",
    // pt_ptr already in r8, syscall arg 5 register
    // Move child_tid_ptr into syscall arg 4 register (our arg 6)
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
    // Move the first arg that went on the stack into rcx (stack_unmap_ptr)
    "mov rcx, [8 + rsp]",
    // Move stack_unmap_ptr onto our new stack
    "mov [rsi], rcx",
    // Move the second arg that went on the stack into rcx (stack_sz)
    "mov rcx, [16 + rsp]",
    // Move down stack ptr
    "sub rsi, 8",
    // Move stack_sz onto the new stack
    "mov [rsi], rcx",
    // Make clone syscall
    "syscall",
    // Check if the syscall return vaulue is 0
    "test eax, eax",
    // if not zero, return (we're the calling thread)
    "jnz 1f",
    // Child:
    // Zero the base pointer
    "xor ebp, ebp",
    // Pop the stack len off the provided stack into callee saved register
    "pop r13",
    // Pop the stack_ptr off the provided stack into another callee saved register
    "pop r12",
    // Pop the start fn args off the provided stack into rdi
    "pop rdi",
    // Call the function we saved in r9, rdi first arg
    "call r9",
    // Zero rax (function return, we don't care)
    "xor rax, rax",
    // Move MUNMAP syscall into ax
    "mov al, 11",
    // Stack ptr as the first arg
    "mov rdi, r12",
    // Stack len as the second arg
    "mov rsi, r13",
    // Syscall, unmapping the stack
    "syscall",
    // Clear the output register, we can't use the return value anyway
    "xor eax,eax",
    // Move EXIT syscall nr into ax
    "mov al, 60",
    // Set exit code for the thread to 0
    "mov rdi, 0",
    // Make exit syscall
    "syscall",
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
    // Move word into double-word arg 1 syscall register
    "uxtw x0, w2",
    // Zero ptid, not using, arg 3 syscall register
    "eor x2, x2, x2",
    // Move pt_ptr into arg 4 syscall register
    "mov x3, x4",
    // Move child tid ptr into arg 5 syscall register
    "mov x4, x5",
    // Move clone syscall nr into nr register
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
    // Move stack_ptr into syscall arg 1 register, also clearing func return value
    "mov x0, x20",
    // Move stack_sz into syscall arg 2 register
    "mov x1, x21",
    // Move munmap syscall nr into syscall nr register
    "mov x8, #215",
    // Make munmap syscall, unmap stack
    "svc #0",
    // Set exit code to 0
    "mov x0, 0",
    // Move exit syscall nr into syscall nr register
    "mov x8, #93",
    // Make exit syscall, thread is done
    "svc #0",
);

#[inline]
#[must_use]
fn get_tls_ptr() -> *mut ThreadLocalStorage {
    let mut output: usize;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {x}, fs:0", x = out(reg) output);
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("mrs {x}, tpidr_el0", x = out(reg) output);
    }
    output as _
}

/// Panic handler
#[panic_handler]
pub fn on_panic(info: &core::panic::PanicInfo) -> ! {
    let tls = get_tls_ptr();
    unsafe {
        // Safety: All threads have TLS set, including the main thread
        let stack_info = tls.read();
        // The main thread does not have stack_info set
        if let Some(stack_dealloc) = stack_info.thread_stack_info() {
            // Dealloc tls, we're done with it, we're panicking so just clean everything up.
            dealloc(tls.cast(), Layout::new::<ThreadLocalStorage>());
            let map_ptr = stack_dealloc.stack_addr;
            let map_len = stack_dealloc.stack_sz;
            let sync = stack_dealloc.sync_ptr as *mut AtomicBool;
            let should_dealloc = sync
                .as_ref()
                .unwrap_unchecked()
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err();
            if should_dealloc {
                // The caller has stopped waiting for a response from this thread.
                // We're responsible from cleaning up the shared memory.
                sc::syscall!(SET_TID_ADDRESS, 0);
                dealloc(stack_dealloc.payload_ptr as _, stack_dealloc.payload_layout);
                dealloc(sync.cast(), Layout::new::<AtomicBool>());
                dealloc(
                    stack_dealloc.futex_ptr as *mut u8,
                    Layout::new::<AtomicU32>(),
                );
            }
            // We need to be able to unmap the thread's own stack, we can't use the stack anymore after that
            // so it needs to be done in asm.
            // With the stack_ptr and stack_len in rdi/x0 and rsi/x1, respectively we can call mmap then
            // exit the thread
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!(
            // Call munmap, all args are provided in this macro call.
            "syscall",
            // Zero eax from munmap ret value
            "xor eax, eax",
            // Move exit into ax
            "mov al, 60",
            // Exit code 0 from thread.
            "mov rdi, 0",
            // Call exit, no return
            "syscall",
            in("rax") MUNMAP,
            in("rdi") map_ptr,
            in("rsi") map_len,
            options(nostack, noreturn)
            );
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!(
            // Make munmap syscall, unmap stack
            "svc #0",
            // Move exit code 0 into arg 1 register
            "mov x0, 0",
            // Move exit syscall nr into syscall nr register
            "mov x8, #93",
            // Make exit syscall, thread is done
            "svc #0",
            in("x8") MUNMAP,
            in("x0") map_ptr,
            in("x1") map_len,
            options(nostack, noreturn)
            );
        } else {
            unix_print::unix_eprintln!("Main thread panicked: {}", info);
            rusl::process::exit(1)
        }
    }
}
