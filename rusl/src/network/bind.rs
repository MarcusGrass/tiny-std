use sc::syscall;

use crate::platform::{Fd, SocketAddressInet, SocketArgUnix};
use crate::Result;

/// Bind the unix-socket with the fd `sock_fd` to the address `socket_address`
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/connect.2.html)
/// Similar to `connect` but on the 'server'-side
/// # Errors
/// See above
#[inline]
pub fn bind_unix(sock_fd: Fd, socket_address: &SocketArgUnix) -> Result<()> {
    let res = unsafe {
        syscall!(
            BIND,
            sock_fd.0,
            core::ptr::addr_of!(socket_address.addr),
            socket_address.addr_len
        )
    };
    bail_on_below_zero!(res, "`BIND` syscall failed");
    Ok(())
}

/// Bind the tcp-socket with the fd `sock_fd` to the address `socket_address`
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/connect.2.html)
/// Similar to `connect` but on the 'server'-side
/// # Errors
/// See above
pub fn bind_inet(sock_fd: Fd, socket_address_inet: &SocketAddressInet) -> Result<()> {
    let res = unsafe {
        syscall!(
            BIND,
            sock_fd.0,
            socket_address_inet as *const SocketAddressInet,
            SocketAddressInet::LENGTH
        )
    };
    bail_on_below_zero!(res, "`BIND` syscall failed");
    Ok(())
}
