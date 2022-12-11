use sc::syscall;

use crate::process::CloneFlags;

/// Unshare the properties specified by `CloneFlags` with other processes.
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/unshare.2.html)
/// # Errors
/// See above
pub fn unshare(flags: CloneFlags) -> crate::Result<()> {
    let res = unsafe { syscall!(UNSHARE, flags.bits()) };
    bail_on_below_zero!(res, "`UNSHARE` syscall failed");
    Ok(())
}
