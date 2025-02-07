use sc::syscall;

use crate::error::Result;
use crate::platform::{Fd, SocketAddressInet, SocketArgUnix};

/// Connect the unix-socket with the fd `sock_fd` to the address `socket_address`
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/connect.2.html)
/// Similar to `bind` but on the 'client'-side
/// # Errors
/// See above
#[inline]
pub fn connect_unix(sock_fd: Fd, socket_address: &SocketArgUnix) -> Result<()> {
    let res = unsafe {
        syscall!(
            CONNECT,
            sock_fd.0,
            core::ptr::addr_of!(socket_address.addr),
            socket_address.addr_len
        )
    };
    bail_on_below_zero!(res, "`CONNECT` syscall failed");
    Ok(())
}

/// Connect the tcp-socket with the fd `sock_fd` to the address `socket_address`
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/connect.2.html)
/// Similar to `bind` but on the 'client'-side
/// # Errors
/// See above
#[inline]
pub fn connect_inet(sock_fd: Fd, addr: &SocketAddressInet) -> Result<()> {
    let res = unsafe {
        syscall!(
            CONNECT,
            sock_fd.0,
            core::ptr::from_ref::<SocketAddressInet>(addr),
            SocketAddressInet::LENGTH
        )
    };
    bail_on_below_zero!(res, "`CONNECT` syscall failed");
    Ok(())
}
