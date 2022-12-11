use sc::syscall;

use crate::platform::PidT;

/// Forks a process returning the pid of the spawned child to the parent,
/// while the child gets 0 returned back.
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/fork.2.html).
/// # Errors
/// See above
/// # Safety
/// Extremely unsafe, reading the documentation thoroughly is recommended for proper usage
#[cfg(target_arch = "x86_64")]
pub unsafe fn fork() -> crate::Result<PidT> {
    let res = syscall!(FORK);
    bail_on_below_zero!(res, "`FORK` syscall failed");
    Ok(res as PidT)
}

/// Fork isn't implemented for aarch64, we're substituting with a clone call here
/// # Errors
/// See above
/// # Safety
/// See above
#[cfg(target_arch = "aarch64")]
pub unsafe fn fork() -> crate::Result<PidT> {
    // `SIGCHLD` is mandatory on aarch64 if mimicking fork it seems
    let cflgs = crate::platform::SIGCHLD;
    let res = syscall!(CLONE, cflgs, 0, 0, 0, 0);
    bail_on_below_zero!(res, "`CLONE` syscall failed");
    Ok(res as i32)
}
