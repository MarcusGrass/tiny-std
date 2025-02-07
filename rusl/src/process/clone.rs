use sc::syscall;

use crate::platform::{Clone3Args, CloneArgs, PidT};
use crate::Result;

/// Forks a process returning the pid of the spawned child to the parent,
/// while the child gets 0 returned back.
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/fork.2.html).
/// # Errors
/// See above
/// # Safety
/// Extremely unsafe, reading the documentation thoroughly is recommended for proper usage
#[cfg(target_arch = "x86_64")]
pub unsafe fn fork() -> Result<PidT> {
    let res = syscall!(FORK);
    bail_on_below_zero!(res, "`FORK` syscall failed");
    // We're trusting the syscall [API here](https://man7.org/linux/man-pages/man2/fork.2.html#RETURN_VALUE)
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    Ok(res as PidT)
}

/// Fork isn't implemented for aarch64, we're substituting with a clone call here
/// # Errors
/// See above
/// # Safety
/// See above
#[cfg(target_arch = "aarch64")]
pub unsafe fn fork() -> Result<PidT> {
    // `SIGCHLD` is mandatory on aarch64 if mimicking fork it seems
    let cflgs = crate::platform::SignalKind::SIGCHLD;
    let res = syscall!(CLONE, cflgs.0 .0, 0, 0, 0, 0);
    bail_on_below_zero!(res, "`CLONE` syscall failed");
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    Ok(res as i32)
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
    // Argument order differs per architecture
    let flags = args.flags.bits() | args.exit_signal.0.into_u64();
    #[cfg(target_arch = "x86_64")]
    let res = unsafe {
        syscall!(
            CLONE,
            flags,
            args.stack,
            args.parent_tid,
            args.child_tid,
            args.tls
        )
    };
    #[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
    let res = {
        syscall!(
            CLONE,
            flags,
            args.stack,
            args.parent_tid,
            args.tls,
            args.child_tid
        )
    };
    bail_on_below_zero!(res, "`CLONE` syscall failed");
    // We're trusting the syscall [API here](https://man7.org/linux/man-pages/man2/clone.2.html#RETURN_VALUE)
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    Ok(res as PidT)
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
    let res = syscall!(CLONE3, core::ptr::from_mut::<Clone3Args>(clone_args), SIZE);
    bail_on_below_zero!(res, "`CLONE3` syscall failed");
    Ok(res as u64)
}
