use sc::syscall;
use unix_print::unix_eprintln;
use crate::platform::{Fd, PidT, TidT};

use crate::Result;

transparent_bitflags!(
    pub struct CloneFlags: u64 {
        const CLONE_VM = linux_rust_bindings::CLONE_VM as u64;
        const CLONE_FS = linux_rust_bindings::CLONE_FS as u64;
        const CLONE_FILES = linux_rust_bindings::CLONE_FILES as u64;
        const CLONE_SIGHAND = linux_rust_bindings::CLONE_SIGHAND as u64;
        const CLONE_PIDFD = linux_rust_bindings::CLONE_PIDFD as u64;
        const CLONE_PTRACE = linux_rust_bindings::CLONE_PTRACE as u64;
        const CLONE_VFORK = linux_rust_bindings::CLONE_VFORK as u64;
        const CLONE_PARENT = linux_rust_bindings::CLONE_PARENT as u64;
        const CLONE_THREAD = linux_rust_bindings::CLONE_THREAD as u64;
        const CLONE_NEWNS = linux_rust_bindings::CLONE_NEWNS as u64;
        const CLONE_SYSVSEM = linux_rust_bindings::CLONE_SYSVSEM as u64;
        const CLONE_SETTLS = linux_rust_bindings::CLONE_SETTLS as u64;
        const CLONE_PARENT_SETTID = linux_rust_bindings::CLONE_PARENT_SETTID as u64;
        const CLONE_CHILD_CLEARTID = linux_rust_bindings::CLONE_CHILD_CLEARTID as u64;
        const CLONE_DETACHED = linux_rust_bindings::CLONE_DETACHED as u64;
        const CLONE_UNTRACED = linux_rust_bindings::CLONE_UNTRACED as u64;
        const CLONE_CHILD_SETTID = linux_rust_bindings::CLONE_CHILD_SETTID as u64;
        const CLONE_NEWCGROUP = linux_rust_bindings::CLONE_NEWCGROUP as u64;
        const CLONE_NEWUTS = linux_rust_bindings::CLONE_NEWUTS as u64;
        const CLONE_NEWIPC = linux_rust_bindings::CLONE_NEWIPC as u64;
        const CLONE_NEWUSER = linux_rust_bindings::CLONE_NEWUSER as u64;
        const CLONE_NEWPID = linux_rust_bindings::CLONE_NEWPID as u64;
        const CLONE_NEWNET = linux_rust_bindings::CLONE_NEWNET as u64;
        const CLONE_IO = linux_rust_bindings::CLONE_IO as u64;
    }
);

#[derive(Debug, Copy, Clone)]
pub struct CloneArgs {
    /// Flags to apply
    flags: CloneFlags,
    /// Pointer to where the child tid should be stored in the child's memory
    child_tid: *mut TidT,
    /// Pointer to where the child tid should be stored in the parent's memory
    parent_tid: *mut TidT,
    /// Pointer to the start of the new thread's stack
    stack: *mut u8,
    /// No idea
    tls: usize,
}

impl CloneArgs {
    #[must_use]
    pub fn new(flags: CloneFlags) -> Self {
        Self {
            flags,
            child_tid: core::ptr::null_mut(),
            parent_tid: core::ptr::null_mut(),
            stack: core::ptr::null_mut(),
            tls: 0,
        }
    }

    /// Where to store the child `Tid` in the child's memory, this should be passed to the child
    #[inline]
    pub fn set_child_tid(&mut self, child_tid_ptr: *mut TidT) -> &mut Self {
        self.child_tid = child_tid_ptr;
        self
    }

    /// Where to store the child `Tid` in the parent's memory, this should be passed to the parent
    #[inline]
    pub fn set_parent_tid(&mut self, parent_tid: *mut TidT) -> &mut Self {
        self.parent_tid = parent_tid;
        self
    }

    /// Set the allocated thread stack area, take care of how that memory is handled
    #[inline]
    pub fn set_stack(&mut self, stack: &mut [u8]) -> &mut Self {
        self.stack = stack.as_mut_ptr();
        self
    }

    #[inline]
    pub fn set_tls(&mut self, tls: usize) -> &mut Self {
        self.tls = tls;
        self
    }

}

/// Invoke the clone syscall with the provided `CloneArgs`.
/// This function is inherently unsafe as poor arguments will cause memory unsafety,
/// for example specifying `CLONE_VM` with a null stack pointer.
/// Please read the [Linux documentation for details](https://man7.org/linux/man-pages/man2/clone.2.html)
/// # Errors
/// See above
/// # Safety
/// See above
pub unsafe fn clone(args: &CloneArgs) -> Result<PidT> {
    unix_eprintln!("{args:?}");
    // Argument order differs per architecture
    #[cfg(any(target_arch = "x86_64"))]
    let res = unsafe {
        syscall!(CLONE, args.flags.bits(), args.stack, args.parent_tid, args.child_tid, args.tls)
    };
    #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
    let res = {
        syscall!(CLONE, args.flags.bits(), args.stack, args.parent_tid, args.tls, args.child_tid)
    };
    bail_on_below_zero!(res, "`CLONE` syscall failed");
    Ok(res as PidT)
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Clone3Args(linux_rust_bindings::clone_args);

impl Clone3Args {
    /// Create a new instance of clone args, the minimum configuration for it to be valid is
    /// a valid set of clone flags, please see the documentation for `clone3`.
    #[must_use]
    pub fn new(flags: CloneFlags) -> Self {
        Self(
            linux_rust_bindings::clone_args {
                flags: flags.bits(),
                pidfd: 0,
                child_tid: 0,
                parent_tid: 0,
                exit_signal: 0,
                stack: 0,
                stack_size: 0,
                tls: 0,
                set_tid: 0,
                set_tid_size: 0,
                cgroup: 0,
            }
        )
    }

    /// Add additional flags
    #[inline]
    pub fn add_flags(&mut self, flags: CloneFlags) -> &mut Self {
        self.0.flags |= flags.bits();
        self
    }

    /// Remove current flags and replace with the provided ones
    #[inline]
    pub fn set_flags(&mut self, flags: CloneFlags) -> &mut Self {
        self.0.flags = flags.bits();
        self
    }

    /// Provide a pointer to which the OS will write an `fd` which can be awaited
    /// for the spawned process' exit code
    #[inline]
    pub fn set_pid_fd(&mut self, pid_fd_ptr: *mut Fd) -> &mut Self {
        self.0.pidfd = pid_fd_ptr as u64;
        self
    }

    /// Where to store the child `Tid` in the child's memory, this should be passed to the child
    #[inline]
    pub fn set_child_tid(&mut self, child_tid_ptr: *mut TidT) -> &mut Self {
        self.0.child_tid = child_tid_ptr as u64;
        self
    }

    /// Where to store the child `Tid` in the parent's memory, this should be passed to the parent
    #[inline]
    pub fn set_parent_tid(&mut self, parent_tid: *mut TidT) -> &mut Self {
        self.0.parent_tid = parent_tid as u64;
        self
    }

    #[inline]
    pub fn set_exit_signal(&mut self, exit_signal: u64) -> &mut Self {
        self.0.exit_signal = exit_signal;
        self
    }

    /// Set the allocated thread stack area, take care of how that memory is handled
    #[inline]
    pub fn set_stack(&mut self, stack: &mut [u8]) -> &mut Self {
        self.0.stack_size = stack.len() as u64;
        self.0.stack = stack.as_mut_ptr() as u64;
        self
    }

    #[inline]
    pub fn set_tls(&mut self, tls: u64) -> &mut Self {
        self.0.tls = tls;
        self
    }

    #[inline]
    pub fn set_set_tid(&mut self, set_tid: u64) -> &mut Self {
        self.0.set_tid = set_tid;
        self
    }

    #[inline]
    pub fn set_set_tid_size(&mut self, set_tid_size: u64) -> &mut Self {
        self.0.set_tid_size = set_tid_size;
        self
    }

    #[inline]
    pub fn set_cgroup(&mut self, cgroup: u64) -> &mut Self {
        self.0.cgroup = cgroup;
        self
    }
}

/// Clone this process using the specified arguments.
/// Has an extreme risk of unsafety as the args determines among other things, what stack area to use
/// which could almost immediately cause UB regardless if it is left as null or directly specified.
/// See [Linux documentation for correct usage](https://man7.org/linux/man-pages/man2/clone.2.html)
/// # Errors
/// See above
/// # Safety
/// See above
pub unsafe fn clone3(clone_args: &mut Clone3Args) -> Result<u64> {
    const SIZE: usize = core::mem::size_of::<Clone3Args>();
    let res = syscall!(CLONE3, clone_args as *mut Clone3Args, SIZE);
    bail_on_below_zero!(res, "`CLONE3` syscall failed");
    Ok(res as u64)
}
