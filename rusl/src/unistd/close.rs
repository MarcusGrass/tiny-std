use sc::syscall;

use crate::platform::Fd;

/// Attempts to close the provided `Fd`.
/// See the [linux docs for details.](https://man7.org/linux/man-pages/man2/close.2.html)
/// # Errors
/// See above
#[inline]
pub fn close(fd: Fd) -> crate::Result<()> {
    let res = unsafe { syscall!(CLOSE, fd.0) };
    bail_on_below_zero!(res, "Failed `CLOSE` syscall");
    Ok(())
}
