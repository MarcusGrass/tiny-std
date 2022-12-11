#[cfg(test)]
mod test;

use core::mem::MaybeUninit;

use sc::syscall;

use crate::string::unix_str::AsUnixStr;
use crate::platform::{Fd, Stat, AT_EMPTY_PATH, AT_FDCWD};

/// [stat](https://man7.org/linux/man-pages/man2/statx.2.html)
/// Gets file status at the path pointed to by `path`
/// # Errors
/// See above docs
#[inline]
pub fn stat(path: impl AsUnixStr) -> crate::Result<Stat> {
    statat(AT_FDCWD, path)
}

/// [fstat](https://man7.org/linux/man-pages/man2/stat.2.html)
/// Gets file status at the file pointed to by `Fd`
/// # Errors
/// See above docs
pub fn statat(fd: Fd, path: impl AsUnixStr) -> crate::Result<Stat> {
    path.exec_with_self_as_ptr(|ptr| {
        let mut stat = MaybeUninit::uninit();
        let res = unsafe { syscall!(NEWFSTATAT, fd, ptr, stat.as_mut_ptr(), AT_EMPTY_PATH) };
        bail_on_below_zero!(res, "`STAT` syscall failed");
        // Safety:
        // We're relying on the os to not supply a nullptr on success
        Ok(unsafe { stat.assume_init() })
    })
}
