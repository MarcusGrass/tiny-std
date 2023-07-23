use sc::syscall;

use crate::platform::{Fd, NonNegativeI32};
use crate::Result;

/// Marks a socket as passive and allows it to accept incoming connection requests
/// See [Linux documentation for more details](https://man7.org/linux/man-pages/man2/listen.2.html)
/// # Errors
/// See above
#[inline]
pub fn listen(sock_fd: Fd, backlog: NonNegativeI32) -> Result<()> {
    let res = unsafe { syscall!(LISTEN, sock_fd.0, backlog.0) };
    bail_on_below_zero!(res, "`LISTEN` syscall failed");
    Ok(())
}
