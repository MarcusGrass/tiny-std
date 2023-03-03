use crate::string::unix_str::AsUnixStr;
use crate::Result;
use sc::syscall;

pub fn swapon(path: impl AsUnixStr, flags: i32) -> Result<()> {
    path.exec_with_self_as_ptr(|ptr| {
        unsafe {
            let res = syscall!(SWAPON, ptr, flags as usize);
            bail_on_below_zero!(res, "`SWAPON` syscall failed");
        }
        Ok(())
    })
}
