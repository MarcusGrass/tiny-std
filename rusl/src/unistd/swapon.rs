use crate::string::unix_str::UnixStr;
use crate::Result;
use sc::syscall;

/// Mount a swap.
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/swapon.2.html).
/// # Errors
/// See above.
pub fn swapon(path: &UnixStr, flags: i32) -> Result<()> {
    unsafe {
        let res = syscall!(SWAPON, path.as_ptr(), flags);
        bail_on_below_zero!(res, "`SWAPON` syscall failed");
    }
    Ok(())
}
