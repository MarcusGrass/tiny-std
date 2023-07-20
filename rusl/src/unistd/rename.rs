use sc::syscall;

use crate::platform::{Fd, RenameFlags, AT_FDCWD};
use crate::string::unix_str::AsUnixStr;

/// Renames `old_path` to `new_path` overwriting any content at `new_path`
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[allow(clippy::inline_always)]
pub fn rename(old_path: impl AsUnixStr, new_path: impl AsUnixStr) -> crate::Result<()> {
    do_rename_at(AT_FDCWD, old_path, AT_FDCWD, new_path, RenameFlags::empty())
}

/// Renames `old_path` to `new_path` with `flags`.
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[allow(clippy::inline_always)]
pub fn rename_flags(
    old_path: impl AsUnixStr,
    new_path: impl AsUnixStr,
    flags: RenameFlags,
) -> crate::Result<()> {
    do_rename_at(AT_FDCWD, old_path, AT_FDCWD, new_path, flags)
}

/// Renames `old_path` to `new_path` with `flags`.
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[allow(clippy::inline_always)]
pub fn rename_at(
    old_dir_fd: Fd,
    old_path: impl AsUnixStr,
    new_dir_fd: Fd,
    new_path: impl AsUnixStr,
) -> crate::Result<()> {
    do_rename_at(
        old_dir_fd.value(),
        old_path,
        new_dir_fd.value(),
        new_path,
        RenameFlags::empty(),
    )
}

/// Renames `old_path` to `new_path` with `flags`.
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[allow(clippy::inline_always)]
pub fn rename_at2(
    old_dir_fd: Fd,
    old_path: impl AsUnixStr,
    new_dir_fd: Fd,
    new_path: impl AsUnixStr,
    flags: RenameFlags,
) -> crate::Result<()> {
    do_rename_at(
        old_dir_fd.value(),
        old_path,
        new_dir_fd.value(),
        new_path,
        flags,
    )
}

fn do_rename_at(
    old_dir_fd: i32,
    old_path: impl AsUnixStr,
    new_dir_fd: i32,
    new_path: impl AsUnixStr,
    flags: RenameFlags,
) -> crate::Result<()> {
    old_path.exec_with_self_as_ptr(move |old_ptr| {
        new_path.exec_with_self_as_ptr(move |new_ptr| {
            let res = unsafe {
                syscall!(
                    RENAMEAT2,
                    old_dir_fd,
                    old_ptr,
                    new_dir_fd,
                    new_ptr,
                    flags.bits()
                )
            };
            bail_on_below_zero!(res, "`RENAMEAT2` syscall failed`");
            Ok(())
        })
    })
}
