use sc::syscall;

use crate::platform::{AcceptFlags, Fd, SocketArg};
use crate::Result;

/// Accept a new connection and set flags on the new connection's `Fd`
/// Accepted flags are 0, `SOCK_NONBLOCK` an `SOCK_CLOEXEC`
/// The `socket_address` is the peer address, if applicable
/// See [Linux documentation for more details](https://man7.org/linux/man-pages/man2/accept.2.html)
/// # Errors
/// See above
#[inline]
pub fn accept(
    sock_fd: Fd,
    socket_address: Option<&SocketArg>,
    new_socket_type: AcceptFlags,
) -> Result<Fd> {
    let (addr, addr_len) = socket_address.map_or((0, 0), |addr| {
        (core::ptr::addr_of!(addr.addr) as usize, addr.addr_len)
    });
    let res = unsafe {
        syscall!(
            ACCEPT4,
            sock_fd.0,
            addr,
            addr_len,
            new_socket_type.bits().bits().0
        )
    };
    Fd::coerce_from_register(res, "`ACCEPT4` syscall failed")
}
