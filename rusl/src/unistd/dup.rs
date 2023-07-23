use sc::syscall;

use crate::error::Errno;
use crate::platform::{Fd, OpenFlags};

/// Duplicates `old` into `new` overwriting `new` if exists.
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/dup.2.html)
/// # Errors
/// See above
#[inline]
pub fn dup2(old: Fd, new: Fd) -> crate::Result<()> {
    dup3(old, new, false)
}

/// Duplicates `old` into `new` overwriting `new` if exists, with flags
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/dup.2.html)
/// # Errors
/// See above
pub fn dup3(old: Fd, new: Fd, cloexec: bool) -> crate::Result<()> {
    loop {
        let res = unsafe { syscall!(DUP3, old.0, new.0, if cloexec { OpenFlags::O_CLOEXEC.0.0 } else { 0 }) };
        // Trusting the systall [API](https://man7.org/linux/man-pages/man2/dup.2.html#RETURN_VALUE)
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        if res as i32 == Errno::EBUSY.raw() {
            continue;
        }
        bail_on_below_zero!(res, "`DUP3` syscall failed");
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::{NonNegativeI32, STDIN};
    use super::*;

    #[test]
    fn can_dup_smoke() {
        dup2(STDIN, NonNegativeI32::comptime_checked_new(999)).unwrap();
        dup3(NonNegativeI32::comptime_checked_new(999), NonNegativeI32::comptime_checked_new(1000), true).unwrap();
    }
}
