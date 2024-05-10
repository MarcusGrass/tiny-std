use sc::syscall;

use crate::platform::{Fd, IoSlice};

/// Attempts to write bytes from the buffer to the corresponding fd.
/// See [linux syscall docs](https://man7.org/linux/man-pages/man2/write.2.html)
/// # Errors
/// See above for possible errors
#[inline]
pub fn write(fd: Fd, buf: &[u8]) -> crate::Result<usize> {
    let res = unsafe { syscall!(WRITE, fd.0, buf.as_ptr(), buf.len()) };
    bail_on_below_zero!(res, "`WRITE` syscall failed");
    Ok(res)
}

/// Attempts to write bytes from the ioslice buffers corresponding fd.
/// See [linux syscall docs](https://man7.org/linux/man-pages/man2/writev.2.html)
/// # Errors
/// See above for possible errors
pub fn writev(fd: Fd, io: &[IoSlice]) -> crate::Result<usize> {
    let res = unsafe { syscall!(WRITEV, fd.0, io.as_ptr(), io.len()) };
    bail_on_below_zero!(res, "`WRITEV` syscall failed");
    Ok(res)
}
