use sc::syscall;

use crate::platform::{Fd, IoSliceMut};

/// Attempts to read bytes up to the buffer's len into the buffer from the provided `Fd`
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/read.2.html)
/// # Errors
/// See above link
#[inline]
pub fn read(fd: Fd, buf: &mut [u8]) -> crate::Result<usize> {
    let res = unsafe { syscall!(READ, fd.0, buf.as_mut_ptr(), buf.len()) };
    bail_on_below_zero!(res, "`READ` syscall failed");
    Ok(res)
}

/// Attempts to read bytes into the provided [`IoSliceMut`]'s from the provided `Fd`
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/read.2.html)
/// # Errors
/// See above link
pub fn readv(fd: Fd, io: &mut [IoSliceMut]) -> crate::Result<usize> {
    let res = unsafe { syscall!(READV, fd.0, io.as_mut_ptr(), io.len()) };
    bail_on_below_zero!(res, "`READV` syscall failed");
    Ok(res)
}
