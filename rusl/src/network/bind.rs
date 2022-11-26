use sc::syscall;

use crate::network::SocketArg;
use crate::platform::Fd;
use crate::Result;

/// Bind the socket with the fd `sock_fd` to the address `socket_address`
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/connect.2.html)
/// Similar to `connect` but on the 'server'-side
/// # Errors
/// See above
#[inline]
pub fn bind(sock_fd: Fd, socket_address: &SocketArg) -> Result<()> {
    let res = unsafe {
        syscall!(
            BIND,
            sock_fd,
            core::ptr::addr_of!(socket_address.addr),
            socket_address.addr_len
        )
    };
    bail_on_below_zero!(res, "`BIND` syscall failed");
    Ok(())
}
