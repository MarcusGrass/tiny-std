use crate::platform::{FcntlFileStatusCmd, Fd, OpenFlags};
use crate::Result;
use sc::syscall;

/// Get the file access mode and status flags
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/fcntl.2.html)
/// # Errors
/// See above, relates to a bad `fd`
pub fn fcntl_get_file_status(fd: Fd) -> Result<OpenFlags> {
    let res = unsafe { syscall!(FCNTL, fd, FcntlFileStatusCmd::Get.into_cmd()) };
    bail_on_below_zero!(res, "`FCNTL` syscall failed");
    Ok(OpenFlags::from(res as i32))
}

/// Set file status flags, access mode and creation flags are valid, but ignored
/// See the [linux documentation for details](https://man7.org/linux/man-pages/man2/fcntl.2.html)
/// # Errors
/// See above
pub fn fcntl_set_file_status(fd: Fd, flag: OpenFlags) -> Result<()> {
    let res = unsafe { syscall!(FCNTL, fd, FcntlFileStatusCmd::Set.into_cmd(), flag.bits()) };
    bail_on_below_zero!(res, "`FCNTL` syscall failed");
    Ok(())
}
