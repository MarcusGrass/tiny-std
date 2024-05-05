use core::mem::MaybeUninit;
use sc::syscall;

use crate::platform::{Fd, SocketAddressInet, SocketAddressUnix, SocketArgUnix, SocketFlags};
use crate::Result;

/// Accept a new unix-connection and set flags on the new connection's `Fd`
/// Accepted flags are 0, `SOCK_NONBLOCK` an `SOCK_CLOEXEC`
/// See [Linux documentation for more details](https://man7.org/linux/man-pages/man2/accept.2.html)
/// # Errors
/// See above
#[inline]
pub fn accept_unix(sock_fd: Fd, flags: SocketFlags) -> Result<(Fd, SocketArgUnix)> {
    let mut addr = MaybeUninit::zeroed();
    let mut addr_len = core::mem::size_of::<SocketAddressUnix>();
    let res = unsafe {
        syscall!(
            ACCEPT4,
            sock_fd.0,
            core::ptr::addr_of_mut!(addr),
            core::ptr::addr_of_mut!(addr_len),
            flags.0
        )
    };
    let fd = Fd::coerce_from_register(res, "`ACCEPT4` syscall failed")?;
    unsafe {
        Ok((
            fd,
            SocketArgUnix {
                addr: addr.assume_init(),
                addr_len,
            },
        ))
    }
}

/// Accept a new tcp-connection and set flags on the new connection's `Fd`
/// Accepted flags are 0, `SOCK_NONBLOCK` an `SOCK_CLOEXEC`
/// See [Linux documentation for more details](https://man7.org/linux/man-pages/man2/accept.2.html)
/// # Errors
/// See above
#[inline]
pub fn accept_inet(sock_fd: Fd, flags: SocketFlags) -> Result<(Fd, SocketAddressInet)> {
    let mut addr = MaybeUninit::zeroed();
    let mut addr_len = core::mem::size_of::<SocketAddressUnix>();
    let res = unsafe {
        syscall!(
            ACCEPT4,
            sock_fd.0,
            core::ptr::addr_of_mut!(addr),
            core::ptr::addr_of_mut!(addr_len),
            flags.0
        )
    };
    let fd = Fd::coerce_from_register(res, "`ACCEPT4` syscall failed")?;
    unsafe { Ok((fd, addr.assume_init())) }
}
