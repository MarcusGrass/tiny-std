use crate::platform::{FcntlFileStatusCmd, Fd, OpenFlags};
use crate::Result;
use sc::syscall;

/// Get the file access mode and status flags
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/fcntl.2.html)
/// # Errors
/// See above, relates to a bad `fd`
pub fn fcntl_get_file_status(fd: Fd) -> Result<OpenFlags> {
    let res = unsafe { syscall!(FCNTL, fd.0, FcntlFileStatusCmd::Get.into_cmd()) };
    Ok(OpenFlags(Fd::coerce_from_register(
        res,
        "`FCNTL` syscall failed",
    )?))
}

/// Set file status flags, access mode and creation flags are valid, but ignored
/// See the [linux documentation for details](https://man7.org/linux/man-pages/man2/fcntl.2.html)
/// # Errors
/// See above
pub fn fcntl_set_file_status(fd: Fd, flag: OpenFlags) -> Result<()> {
    let res = unsafe {
        syscall!(
            FCNTL,
            fd.0,
            FcntlFileStatusCmd::Set.into_cmd(),
            flag.bits().0
        )
    };
    bail_on_below_zero!(res, "`FCNTL` syscall failed");
    Ok(())
}
