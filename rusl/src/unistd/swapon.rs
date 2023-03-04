use crate::string::unix_str::AsUnixStr;
use crate::Result;
use sc::syscall;

/// Mount a swap.
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/swapon.2.html).
/// # Errors
/// See above.
pub fn swapon(path: impl AsUnixStr, flags: i32) -> Result<()> {
    path.exec_with_self_as_ptr(|ptr| {
        unsafe {
            let res = syscall!(SWAPON, ptr, flags as usize);
            bail_on_below_zero!(res, "`SWAPON` syscall failed");
        }
        Ok(())
    })
}
