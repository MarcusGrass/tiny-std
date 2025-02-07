use sc::syscall;

use crate::platform::{Fd, RenameFlags, AT_FDCWD};
use crate::string::unix_str::UnixStr;

/// Renames `old_path` to `new_path` overwriting any content at `new_path`
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[expect(clippy::inline_always)]
pub fn rename(old_path: &UnixStr, new_path: &UnixStr) -> crate::Result<()> {
    do_rename_at(AT_FDCWD, old_path, AT_FDCWD, new_path, RenameFlags::empty())
}

/// Renames `old_path` to `new_path` with `flags`.
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[expect(clippy::inline_always)]
pub fn rename_flags(
    old_path: &UnixStr,
    new_path: &UnixStr,
    flags: RenameFlags,
) -> crate::Result<()> {
    do_rename_at(AT_FDCWD, old_path, AT_FDCWD, new_path, flags)
}

/// Renames `old_path` to `new_path` with `flags`.
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/rename.2.html#ERRORS)
#[inline(always)]
#[expect(clippy::inline_always)]
pub fn rename_at(
    old_dir_fd: Fd,
    old_path: &UnixStr,
    new_dir_fd: Fd,
    new_path: &UnixStr,
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
#[expect(clippy::inline_always)]
pub fn rename_at2(
    old_dir_fd: Fd,
    old_path: &UnixStr,
    new_dir_fd: Fd,
    new_path: &UnixStr,
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
    old_path: &UnixStr,
    new_dir_fd: i32,
    new_path: &UnixStr,
    flags: RenameFlags,
) -> crate::Result<()> {
    let res = unsafe {
        syscall!(
            RENAMEAT2,
            old_dir_fd,
            old_path.as_ptr(),
            new_dir_fd,
            new_path.as_ptr(),
            flags.bits()
        )
    };
    bail_on_below_zero!(res, "`RENAMEAT2` syscall failed`");
    Ok(())
}
