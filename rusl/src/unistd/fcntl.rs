use sc::syscall;
use crate::platform::{FcntlFileStatusCmd, Fd, OpenFlags};
use crate::Result;

pub fn fcntl_get_file_status(fd: Fd, flag: OpenFlags) -> Result<OpenFlags> {
    let res = unsafe {syscall!(FCNTL, fd, FcntlFileStatusCmd::Get.into_cmd(), flag.bits())};
    bail_on_below_zero!(res, "`FCNTL` syscall failed");
    Ok(OpenFlags::from(res as i32))
}
pub fn fcntl_set_file_status(fd: Fd, flag: OpenFlags) -> Result<()> {
    let res = unsafe {syscall!(FCNTL, fd, FcntlFileStatusCmd::Set.into_cmd(), flag.bits())};
    bail_on_below_zero!(res, "`FCNTL` syscall failed");
    Ok(())
}
