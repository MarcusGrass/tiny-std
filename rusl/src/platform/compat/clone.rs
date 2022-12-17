use crate::platform::{Fd, SignalKind, TidT};

transparent_bitflags!(
    pub struct CloneFlags: u64 {
        const CLONE_VM = linux_rust_bindings::sched::CLONE_VM as u64;
        const CLONE_FS = linux_rust_bindings::sched::CLONE_FS as u64;
        const CLONE_FILES = linux_rust_bindings::sched::CLONE_FILES as u64;
        const CLONE_SIGHAND = linux_rust_bindings::sched::CLONE_SIGHAND as u64;
        const CLONE_PIDFD = linux_rust_bindings::sched::CLONE_PIDFD as u64;
        const CLONE_PTRACE = linux_rust_bindings::sched::CLONE_PTRACE as u64;
        const CLONE_VFORK = linux_rust_bindings::sched::CLONE_VFORK as u64;
        const CLONE_PARENT = linux_rust_bindings::sched::CLONE_PARENT as u64;
        const CLONE_THREAD = linux_rust_bindings::sched::CLONE_THREAD as u64;
        const CLONE_NEWNS = linux_rust_bindings::sched::CLONE_NEWNS as u64;
        const CLONE_SYSVSEM = linux_rust_bindings::sched::CLONE_SYSVSEM as u64;
        const CLONE_SETTLS = linux_rust_bindings::sched::CLONE_SETTLS as u64;
        const CLONE_PARENT_SETTID = linux_rust_bindings::sched::CLONE_PARENT_SETTID as u64;
        const CLONE_CHILD_CLEARTID = linux_rust_bindings::sched::CLONE_CHILD_CLEARTID as u64;
        const CLONE_DETACHED = linux_rust_bindings::sched::CLONE_DETACHED as u64;
        const CLONE_UNTRACED = linux_rust_bindings::sched::CLONE_UNTRACED as u64;
        const CLONE_CHILD_SETTID = linux_rust_bindings::sched::CLONE_CHILD_SETTID as u64;
        const CLONE_NEWCGROUP = linux_rust_bindings::sched::CLONE_NEWCGROUP as u64;
        const CLONE_NEWUTS = linux_rust_bindings::sched::CLONE_NEWUTS as u64;
        const CLONE_NEWIPC = linux_rust_bindings::sched::CLONE_NEWIPC as u64;
        const CLONE_NEWUSER = linux_rust_bindings::sched::CLONE_NEWUSER as u64;
        const CLONE_NEWPID = linux_rust_bindings::sched::CLONE_NEWPID as u64;
        const CLONE_NEWNET = linux_rust_bindings::sched::CLONE_NEWNET as u64;
        const CLONE_IO = linux_rust_bindings::sched::CLONE_IO as u64;
    }
);

#[derive(Debug, Copy, Clone)]
pub struct CloneArgs {
    /// Flags to apply
    pub(crate) flags: CloneFlags,
    /// Pointer to where the child tid should be stored in the child's memory
    pub(crate) child_tid: *mut TidT,
    /// Pointer to where the child tid should be stored in the parent's memory
    pub(crate) parent_tid: *mut TidT,
    /// Pointer to the start of the new thread's stack
    pub(crate) stack: *mut u8,
    /// No idea
    pub(crate) tls: usize,
    pub(crate) exit_signal: SignalKind,
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
            exit_signal: SignalKind::empty(),
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
    pub fn set_stack(&mut self, stack: *mut u8) -> &mut Self {
        self.stack = stack;
        self
    }

    #[inline]
    pub fn set_tls(&mut self, tls: usize) -> &mut Self {
        self.tls = tls;
        self
    }

    /// Low byte is the exit signal on a clone call
    #[inline]
    pub fn set_exit(&mut self, exit_signal: SignalKind) -> &mut Self {
        self.exit_signal = exit_signal;
        self
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Clone3Args(linux_rust_bindings::sched::clone_args);

impl Clone3Args {
    /// Create a new instance of clone args, the minimum configuration for it to be valid is
    /// a valid set of clone flags, please see the documentation for `clone3`.
    #[must_use]
    pub fn new(flags: CloneFlags) -> Self {
        Self(linux_rust_bindings::sched::clone_args {
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
        })
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
