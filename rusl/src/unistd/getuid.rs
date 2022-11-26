use sc::syscall;

use crate::platform::UidT;

/// Gets the userid of this process
/// See the [linux docs](https://man7.org/linux/man-pages/man2/getuid.2.html) for details
/// # Errors
/// See above
pub fn get_uid() -> crate::Result<UidT> {
    let res = unsafe { syscall!(GETUID) } as i32;
    bail_on_below_zero!(res, "`GETUID` failed");
    Ok(res as UidT)
}
