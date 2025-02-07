use core::mem::MaybeUninit;
use sc::syscall;

use crate::platform::{
    AddressFamily, Fd, SocketAddressInet, SocketAddressUnix, SocketArgUnix, SocketOptions,
};
use crate::Result;

/// Create a socket with the specified `Domain`, `SocketType`, and `protocol`
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/socket.2.html)
/// # Errors
/// See above
#[inline]
pub fn socket(domain: AddressFamily, options: SocketOptions, protocol: i32) -> Result<Fd> {
    let res = unsafe { syscall!(SOCKET, domain.0, options.0, protocol) };
    Fd::coerce_from_register(res, "`SOCKET` syscall failed")
}

/// Get the socket name of the provided Unix socket [`Fd`].
/// See [Linux docs for details](https://man7.org/linux/man-pages/man2/getsockname.2.html)
/// # Errors
/// See above
pub fn get_unix_sock_name(sock_fd: Fd) -> Result<SocketArgUnix> {
    let mut addr = MaybeUninit::zeroed();
    let mut addr_len = core::mem::size_of::<SocketAddressUnix>();
    let res = unsafe {
        syscall!(
            GETSOCKNAME,
            sock_fd.into_usize(),
            core::ptr::addr_of_mut!(addr),
            core::ptr::addr_of_mut!(addr_len)
        )
    };
    bail_on_below_zero!(res, "`GETSOCKNAME` syscall failed");
    unsafe {
        Ok(SocketArgUnix {
            addr: addr.assume_init(),
            addr_len,
        })
    }
}

/// Get the socket name of the provided Inet socket [`Fd`].
/// See [Linux docs for details](https://man7.org/linux/man-pages/man2/getsockname.2.html)
/// # Errors
/// See above
pub fn get_inet_sock_name(sock_fd: Fd) -> Result<SocketAddressInet> {
    let mut addr = MaybeUninit::zeroed();
    let mut addr_len = core::mem::size_of::<SocketAddressInet>();
    let res = unsafe {
        syscall!(
            GETSOCKNAME,
            sock_fd.into_usize(),
            core::ptr::addr_of_mut!(addr),
            core::ptr::addr_of_mut!(addr_len)
        )
    };
    bail_on_below_zero!(res, "`GETSOCKNAME` syscall failed");
    unsafe { Ok(addr.assume_init()) }
}

/// Send a message on a socket, [`crate::unistd::write`] should be prefered if not sending fds.
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/send.2.html)
/// # Errors
/// See above
#[cfg(feature = "alloc")]
pub fn sendmsg(sock_fd: Fd, send: &crate::platform::SendDropGuard, flags: i32) -> Result<usize> {
    let res = unsafe { syscall!(SENDMSG, sock_fd.0, core::ptr::addr_of!(send.msghdr), flags) };
    bail_on_below_zero!(res, "`SENDMSG` syscall failed");
    Ok(res)
}

/// Read a message from a socket, [`crate::unistd::read`] should be prefered if not expecting to receive fds.
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/recv.2.html)
/// # Errors
/// See above
#[cfg(feature = "alloc")]
pub fn recvmsg(sock_fd: Fd, recv: &mut crate::platform::MsgHdrBorrow, flags: i32) -> Result<usize> {
    let res = unsafe {
        syscall!(
            RECVMSG,
            sock_fd.0,
            core::ptr::from_mut::<crate::platform::MsgHdrBorrow>(recv),
            flags
        )
    };
    bail_on_below_zero!(res, "`RECVMSG` syscall failed");
    Ok(res)
}
