use sc::syscall;

use crate::platform::Fd;
use crate::Result;

transparent_bitflags!(
    pub struct Domain: i32 {
        const AF_UNIX = 1;
        const AF_LOCAL = 1;
        const AF_INET = 2;
        const AF_AX25 = 3;
        const AF_IPX = 4;
        const AF_APPLETALK = 5;
        const AF_X25 = 9;
        const AF_INET6 = 10;
        // Faithful representation
        #[allow(non_upper_case_globals)]
        const AF_DECnet = 12;
        const AF_KEY = 15;
        const AF_NETLINK = 16;
        const AF_PACKET = 17;
        const AF_RDS = 21;
        const AF_PPPOX = 24;
        const AF_LLC = 26;
        const AF_CAN = 29;
        const AF_TIPC = 30;
        const AF_BLUETOOTH = 31;
        const AF_ALG = 38;
        const AF_VSOCK = 40;
        const AF_XDP = 44;
    }
);

transparent_bitflags!(
    pub struct SocketType: i32 {
        const SOCK_STREAM = 1;
        const SOCK_DGRAM = 2;
        const SOCK_RAW = 3;
        const SOCK_RDM = 4;
        const SOCK_SEQPACKET = 5;
        /// Deprecated
        const SOCK_PACKET = 10;
        const SOCK_NONBLOCK = crate::platform::O_NONBLOCK;
        const SOCK_CLOEXEC = crate::platform::O_CLOEXEC;
    }
);

/// Create a socket with the specified `Domain`, `SocketType`, and `protocol`
/// See [linux docs for details](https://man7.org/linux/man-pages/man2/socket.2.html)
/// # Errors
/// See above
#[inline]
pub fn socket(domain: Domain, socket_type: SocketType, protocol: i32) -> Result<Fd> {
    let res = unsafe { syscall!(SOCKET, domain.bits(), socket_type.bits(), protocol) };
    bail_on_below_zero!(res, "`SOCKET` syscall failed");
    Ok(res as i32)
}
