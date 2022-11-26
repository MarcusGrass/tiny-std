use sc::syscall;

use crate::Result;

transparent_bitflags!(
    pub struct CloneFlags: u64 {
        const CLONE_VM = 0x100;
        const CLONE_FS = 0x200;
        const CLONE_FILES = 0x400;
        const CLONE_SIGHAND = 0x800;
        const CLONE_PIDFD = 0x1000;
        const CLONE_PTRACE = 0x2000;
        const CLONE_VFORK = 0x4000;
        const CLONE_PARENT = 0x8000;
        const CLONE_THREAD = 0x10000;
        const CLONE_NEWNS = 0x20000;
        const CLONE_SYSVSEM = 0x40000;
        const CLONE_SETTLS = 0x80000;
        const CLONE_PARENT_SETTID = 0x100000;
        const CLONE_CHILD_CLEARTID = 0x200000;
        const CLONE_DETACHED = 0x400000;
        const CLONE_UNTRACED = 0x800000;
        const CLONE_CHILD_SETTID = 0x01000000;
        const CLONE_NEWCGROUP = 0x02000000;
        const CLONE_NEWUTS = 0x04000000;
        const CLONE_NEWIPC = 0x08000000;
        const CLONE_NEWUSER = 0x10000000;
        const CLONE_NEWPID = 0x20000000;
        const CLONE_NEWNET = 0x40000000;
        const CLONE_IO = 0x80000000;
    }
);

#[repr(C)]
pub struct CloneArgs {
    /* Flags bit mask */
    pub flags: CloneFlags,
    /* Where to store PID file descriptor
    (int *) */
    pub pidfd: u64,
    /* Where to store child TID,
    in child's memory (pid_t *) */
    pub child_tid: u64,
    /* Where to store child TID,
    in parent's memory (pid_t *) */
    pub parent_tid: u64,
    /* Signal to deliver to parent on
    child termination */
    pub exit_signal: u64,
    /* Pointer to lowest byte of stack */
    pub stack: u64,
    /* Size of stack */
    pub stack_size: u64,
    /* Location of new TLS */
    pub tls: u64,
    /* Pointer to a pid_t array
    (since Linux 5.5) */
    pub set_tid: u64,
    /* Number of elements in set_tid
    (since Linux 5.5) */
    pub set_tid_size: u64,
    /* File descriptor for target cgroup
    of child (since Linux 5.7) */
    pub cgroup: u64,
}

/// Clone this process using the specified arguments.
/// Has an extreme risk of unsafety as the args determines among other things, what stack area to use
/// which could almost immediately cause UB regardless if it is left as null or directly specified.
/// See [Linux documentation for correct usage](https://man7.org/linux/man-pages/man2/clone.2.html)
/// # Errors
/// See above
/// # Safety
/// See above
pub unsafe fn clone3(clone_args: &mut CloneArgs) -> Result<u64> {
    const SIZE: usize = core::mem::size_of::<CloneArgs>();
    let res = syscall!(CLONE3, clone_args as *mut CloneArgs, SIZE);
    bail_on_below_zero!(res, "`CLONE3` syscall failed");
    Ok(res as u64)
}
