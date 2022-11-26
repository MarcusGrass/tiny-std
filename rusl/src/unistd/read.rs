use sc::syscall;

use crate::platform::Fd;

/// Attempts to read bytes up to the buffer's len into the buffer from the provided `Fd`
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/read.2.html)
/// # Errors
/// See above link
#[inline]
pub fn read(fd: Fd, buf: &mut [u8]) -> crate::Result<usize> {
    let res = unsafe { syscall!(READ, fd, buf.as_mut_ptr(), buf.len()) as i32 };
    bail_on_below_zero!(res, "`READ` syscall failed");
    Ok(res as usize)
}
