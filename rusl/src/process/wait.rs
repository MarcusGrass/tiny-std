use core::ptr::addr_of_mut;

use sc::syscall;

#[derive(Debug, Copy, Clone)]
pub struct WaitPidResult {
    pub pid: i32,
    pub status: i32,
}

/// Waits for the specified process to finish.
/// See [Linux docs for details](https://man7.org/linux/man-pages/man2/wait4.2.html)
/// # Errors
/// See above
pub fn wait_pid(pid: i32, options: i32) -> crate::Result<WaitPidResult> {
    let mut wstatus = 0i32;
    let res = unsafe { syscall!(WAIT4, pid, addr_of_mut!(wstatus), options, 0) };
    bail_on_below_zero!(res, "`WAIT4` syscall failed");
    // We're trusting the syscall [API here](https://man7.org/linux/man-pages/man2/wait4.2.html#RETURN_VALUE)
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    Ok(WaitPidResult {
        pid: res as i32,
        status: wstatus,
    })
}
