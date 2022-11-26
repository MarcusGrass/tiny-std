use sc::syscall;

use crate::platform::UidT;

/// Sets the calling process' `uid` to the provided `uid`
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/setuid.2.html)
/// # Errors
/// See above
#[inline]
pub fn setuid(uid: UidT) -> crate::Result<()> {
    let res = unsafe { syscall!(SETUID, uid) };
    bail_on_below_zero!(res, "`SETUID` syscall failed");
    Ok(())
}
