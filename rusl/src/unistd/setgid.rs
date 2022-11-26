use sc::syscall;

use crate::platform::GidT;

/// Sets the process' group id to the provided `gid`
/// See [Linux documentation for details](https://man7.org/linux/man-pages/man2/setgid.2.html)
/// # Errors
/// See above
#[inline]
pub fn setgid(gid: GidT) -> crate::Result<()> {
    let res = unsafe { syscall!(SETGID, gid) };
    bail_on_below_zero!(res, "`SETGID` syscall failed");
    Ok(())
}
