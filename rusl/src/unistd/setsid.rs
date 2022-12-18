use sc::syscall;

/// Creates a new session id if this process is not the current process group leader.
/// # Errors
/// Eperm if already a process group leader.
#[inline]
pub fn setsid() -> crate::Result<()> {
    let res = unsafe { syscall!(SETSID) };
    bail_on_below_zero!(res, "`SETSID` syscall failed");
    Ok(())
}
