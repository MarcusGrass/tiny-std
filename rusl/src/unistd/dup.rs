use sc::syscall;

use crate::error::Errno;
use crate::platform::{Fd, OpenFlags};

/// Duplicates `old` into `new` overwriting `new` if exists.
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/dup.2.html)
/// # Errors
/// See above
#[inline]
pub fn dup2(old: Fd, new: Fd) -> crate::Result<()> {
    dup3(old, new, OpenFlags::empty())
}

/// Duplicates `old` into `new` overwriting `new` if exists, with flags
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/dup.2.html)
/// # Errors
/// See above
pub fn dup3(old: Fd, new: Fd, flags: OpenFlags) -> crate::Result<()> {
    loop {
        let res = unsafe { syscall!(DUP3, old, new, flags.bits()) };
        if res as i32 == Errno::EBUSY.raw() {
            continue;
        }
        bail_on_below_zero!(res, "`DUP3` syscall failed");
        return Ok(());
    }
}
