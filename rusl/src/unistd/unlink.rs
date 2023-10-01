use sc::syscall;

use crate::platform::{Fd, AT_FDCWD, AT_REMOVEDIR};
use crate::string::unix_str::UnixStr;

#[derive(Debug, Copy, Clone)]
pub struct UnlinkFlags(i32);

impl UnlinkFlags {
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    #[must_use]
    pub const fn at_removedir() -> Self {
        Self(AT_REMOVEDIR.value())
    }
}

/// Taking the liberty of using `unlinkat` for both implementations, effectively meaning
/// that `CWD` is the base if the path isn't absolute
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn unlink(path: &UnixStr) -> crate::Result<()> {
    unlink_flags(path, UnlinkFlags::empty())
}

/// Unlink with a path from `CWD` and the provided flags. Flags are either 0 or `AT_REMOVEDIR`.
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn unlink_flags(path: &UnixStr, flags: UnlinkFlags) -> crate::Result<()> {
    do_unlink(AT_FDCWD, path, flags)
}

/// Taking the liberty of using `unlinkat` for both implementations, effectively meaning
/// that `CWD` is the base if the path isn't absolute. Flags are either 0 or `AT_REMOVEDIR`
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn unlink_at(dir_fd: Fd, path: &UnixStr, flags: UnlinkFlags) -> crate::Result<()> {
    do_unlink(dir_fd.0, path, flags)
}

#[inline(always)]
#[allow(clippy::inline_always)]
fn do_unlink(dir_fd: i32, path: &UnixStr, flags: UnlinkFlags) -> crate::Result<()> {
    let res = unsafe { syscall!(UNLINKAT, dir_fd, path.as_ptr(), flags.0) };
    bail_on_below_zero!(res, "`UNLINKAT` syscall failed");
    Ok(())
}

/// Taking the liberty of using `unlinkat` for both implementations, effectively meaning
/// that `CWD` is the base if the path isn't absolute. Flags are either 0 or `AT_REMOVEDIR`
/// "unlink, unlinkat - delete a name and possibly the file it refers to"[docs](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn rmdir(dir_fd: Fd) -> crate::Result<()> {
    let res = unsafe { syscall!(UNLINKAT, dir_fd.0, 0, AT_REMOVEDIR.0) };
    bail_on_below_zero!(res, "`UNLINKAT` syscall failed");
    Ok(())
}
