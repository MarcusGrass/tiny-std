use sc::syscall;

use crate::network::{SocketArg, SocketType};
use crate::platform::Fd;
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
    new_socket_type: SocketType,
) -> Result<Fd> {
    let (addr, addr_len) = socket_address.map_or((0, 0), |addr| {
        (core::ptr::addr_of!(addr.addr) as usize, addr.addr_len)
    });
    let res = unsafe { syscall!(ACCEPT4, sock_fd, addr, addr_len, new_socket_type.bits()) };
    bail_on_below_zero!(res, "`ACCEPT4` syscall failed");
    Ok(res as i32)
}
