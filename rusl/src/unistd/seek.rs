use crate::platform::{Fd, NonNegativeI32, OffT};
use crate::Error;
use sc::syscall;

transparent_bitflags! {
    pub struct Whence: NonNegativeI32 {
        const DEFAULT = NonNegativeI32::ZERO;
        const SET = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fs::SEEK_SET);
        const CUR = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fs::SEEK_CUR);
        const END = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fs::SEEK_END);
        const DATA = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fs::SEEK_DATA);
        const HOLE = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fs::SEEK_HOLE);
    }
}

/// Seek a fd
/// See details in the [linux docs here](https://man7.org/linux/man-pages/man2/lseek.2.html)
/// # Errors
/// See above
#[expect(clippy::cast_possible_wrap)]
pub fn lseek(fd: Fd, off_t: OffT, whence: Whence) -> Result<OffT, Error> {
    unsafe {
        let res = syscall!(LSEEK, fd.0, off_t, whence.bits().0);
        bail_on_below_zero!(res, "`LSEEK syscall failed`");
        Ok(res as OffT)
    }
}
