use sc::syscall;

use crate::platform::PidT;

/// Sets process group id `grp_pid` to the process specified py `set_pid`
/// See [Linux documentation for detials](https://man7.org/linux/man-pages/man2/getpgrp.2.html)
/// # Errors
/// See above
#[inline]
pub fn setpgid(set_pid: PidT, grp_pid: PidT) -> crate::Result<()> {
    let res = unsafe { syscall!(SETPGID, set_pid, grp_pid) };
    bail_on_below_zero!(res, "`SETPGID` syscall failed");
    Ok(())
}
