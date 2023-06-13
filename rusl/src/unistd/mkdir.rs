use sc::syscall;

use crate::platform::{Fd, Mode, AT_FDCWD};
use crate::string::unix_str::AsUnixStr;
use crate::Result;

/// Create a directory named `path`
/// See [linux documentation for details](https://man7.org/linux/man-pages/man2/mkdir.2.html)
/// # Errors
/// See above
#[inline]
pub fn mkdir(path: impl AsUnixStr, mode: Mode) -> Result<()> {
    do_mkdir(AT_FDCWD, path, mode)
}

/// Create a directory named `path` with `dir_fd` as the root directory
/// See [linux documentation for details](https://man7.org/linux/man-pages/man2/mkdir.2.html)
/// # Errors
/// See above
pub fn mkdir_at(dir_fd: Fd, path: impl AsUnixStr, mode: Mode) -> Result<()> {
    do_mkdir(dir_fd.0, path, mode)
}

#[inline(always)]
#[allow(clippy::inline_always)]
fn do_mkdir(dir_fd: i32, path: impl AsUnixStr, mode: Mode) -> Result<()> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(MKDIRAT, dir_fd, ptr, mode.bits()) };
        bail_on_below_zero!(res, "`MKDIRAT` syscall failed");
        Ok(())
    })
}
