use sc::syscall;

use crate::string::unix_str::AsUnixStr;
use crate::platform::{Fd, AT_FDCWD, AT_REMOVEDIR};

/// Taking the liberty of using `unlinkat` for both implementations, effectively meaning
/// that `CWD` is the base if the path isn't absolute
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn unlink(path: impl AsUnixStr) -> crate::Result<()> {
    unlink_flags(path, 0)
}

/// Unlink with a path from `CWD` and the provided flags. Flags are either 0 or `AT_REMOVEDIR`.
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn unlink_flags(path: impl AsUnixStr, flags: i32) -> crate::Result<()> {
    unlink_at(AT_FDCWD, path, flags)
}

/// Taking the liberty of using `unlinkat` for both implementations, effectively meaning
/// that `CWD` is the base if the path isn't absolute. Flags are either 0 or `AT_REMOVEDIR`
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn unlink_at(dir_fd: Fd, path: impl AsUnixStr, flags: i32) -> crate::Result<()> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(UNLINKAT, dir_fd, ptr, flags) };
        bail_on_below_zero!(res, "`UNLINKAT` syscall failed");
        Ok(())
    })
}

/// Taking the liberty of using `unlinkat` for both implementations, effectively meaning
/// that `CWD` is the base if the path isn't absolute. Flags are either 0 or `AT_REMOVEDIR`
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn rmdir(dir_fd: Fd) -> crate::Result<()> {
    let res = unsafe { syscall!(UNLINKAT, dir_fd, 0, AT_REMOVEDIR) };
    bail_on_below_zero!(res, "`UNLINKAT` syscall failed");
    Ok(())
}
