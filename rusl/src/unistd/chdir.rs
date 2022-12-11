use sc::syscall;

use crate::string::unix_str::AsUnixStr;
use crate::Result;

/// Changes the working directory of the current process
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/chdir.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn chdir<P: AsUnixStr>(path: P) -> Result<()> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(CHDIR, ptr) };
        bail_on_below_zero!(res, "`CHDIR` syscall failed");
        Ok(())
    })
}
