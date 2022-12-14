use sc::syscall;

use crate::platform::Fd;

/// Attempts to write bytes from the buffer to the corresponding fd.
/// See [linux syscall docs](https://man7.org/linux/man-pages/man2/write.2.html)
/// # Errors
/// See above for possible errors
#[inline]
pub fn write(fd: Fd, buf: &[u8]) -> crate::Result<usize> {
    let res = unsafe { syscall!(WRITE, fd, buf.as_ptr(), buf.len()) };
    bail_on_below_zero!(res, "`WRITE` syscall failed");
    Ok(res)
}
