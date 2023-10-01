use sc::syscall;

use crate::string::unix_str::UnixStr;
use crate::Result;

/// Changes the working directory of the current process.
/// See the [Linux docs for details](https://man7.org/linux/man-pages/man2/chdir.2.html)
/// # Errors
/// See above docs
#[inline]
pub fn chdir(path: &UnixStr) -> Result<()> {
    let res = unsafe { syscall!(CHDIR, path.as_ptr()) };
    bail_on_below_zero!(res, "`CHDIR` syscall failed");
    Ok(())
}
