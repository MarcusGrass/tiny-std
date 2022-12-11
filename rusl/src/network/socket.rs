use sc::syscall;

use crate::platform::{AddressFamily, Fd, SocketType};
use crate::Result;

/// Create a socket with the specified `Domain`, `SocketType`, and `protocol`
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/socket.2.html)
/// # Errors
/// See above
#[inline]
pub fn socket(domain: AddressFamily, socket_type: SocketType, protocol: i32) -> Result<Fd> {
    let res = unsafe { syscall!(SOCKET, domain.bits(), socket_type.bits(), protocol) };
    bail_on_below_zero!(res, "`SOCKET` syscall failed");
    Ok(res as i32)
}
