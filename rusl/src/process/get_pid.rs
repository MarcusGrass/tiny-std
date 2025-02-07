use crate::platform::PidT;
use sc::syscall;

/// Get the pid of the calling process
/// See [Linux docs for details](https://man7.org/linux/man-pages/man2/getpid.2.html)
/// Always successful
#[inline]
#[must_use]
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn get_pid() -> PidT {
    let res = unsafe { syscall!(GETPID) };
    res as PidT
}
