use sc::syscall;

use crate::platform::UidT;

/// Gets the userid of this process
/// See the [linux docs](https://man7.org/linux/man-pages/man2/getuid.2.html) for details
/// # Errors
/// See above
pub fn get_uid() -> crate::Result<UidT> {
    let res = unsafe { syscall!(GETUID) };
    bail_on_below_zero!(res, "`GETUID` failed");
    // We're trusting the syscall [API here](https://man7.org/linux/man-pages/man2/getuid.2.html)
    Ok(res as UidT)
}
