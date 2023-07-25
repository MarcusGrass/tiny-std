//! These are not defined in the uapi which is a bit hairy, if they change, that's obviously
/// a problem.
use crate::string::unix_str::AsUnixStr;
use crate::Error;

#[derive(Debug, Copy, Clone)]
pub struct AddressFamily(pub(crate) u16);

impl AddressFamily {
    pub const AF_UNSPEC: Self = Self(0);
    pub const AF_UNIX: Self = Self(1); /* Unix domain sockets 		*/
    pub const AF_LOCAL: Self = Self(1); /* POSIX name for AF_UNIX	*/
    pub const AF_INET: Self = Self(2); /* Internet IP Protocol 	*/
    pub const AF_AX25: Self = Self(3); /* Amateur Radio AX.25 		*/
    pub const AF_IPX: Self = Self(4); /* Novell IPX 			*/
    pub const AF_APPLETALK: Self = Self(5); /* AppleTalk DDP 		*/
    pub const AF_NETROM: Self = Self(6); /* Amateur Radio NET/ROM 	*/
    pub const AF_BRIDGE: Self = Self(7); /* Multiprotocol bridge 	*/
    pub const AF_ATMPVC: Self = Self(8); /* ATM PVCs			*/
    pub const AF_X25: Self = Self(9); /* Reserved for X.25 project 	*/
    pub const AF_INET6: Self = Self(10); /* IP version 6			*/
    pub const AF_ROSE: Self = Self(11); /* Amateur Radio X.25 PLP	*/
    #[allow(non_upper_case_globals)]
    pub const AF_DECnet: Self = Self(12); /* Reserved for DECnet project	*/
    pub const AF_NETBEUI: Self = Self(13); /* Reserved for 802.2LLC project*/
    pub const AF_SECURITY: Self = Self(14); /* Security callback pseudo AF */
    pub const AF_KEY: Self = Self(15); /* PF_KEY key management API */
    pub const AF_NETLINK: Self = Self(16);
    pub const AF_ROUTE: Self = Self(Self::AF_NETLINK.0); /* Alias to emulate 4.4BSD */
    pub const AF_PACKET: Self = Self(17); /* Packet family		*/
    pub const AF_ASH: Self = Self(18); /* Ash				*/
    pub const AF_ECONET: Self = Self(19); /* Acorn Econet			*/
    pub const AF_ATMSVC: Self = Self(20); /* ATM SVCs			*/
    pub const AF_RDS: Self = Self(21); /* RDS sockets 			*/
    pub const AF_SNA: Self = Self(22); /* Linux SNA Project (nutters!) */
    pub const AF_IRDA: Self = Self(23); /* IRDA sockets			*/
    pub const AF_PPPOX: Self = Self(24); /* PPPoX sockets		*/
    pub const AF_WANPIPE: Self = Self(25); /* Wanpipe API Sockets */
    pub const AF_LLC: Self = Self(26); /* Linux LLC			*/
    pub const AF_IB: Self = Self(27); /* Native InfiniBand address	*/
    pub const AF_MPLS: Self = Self(28); /* MPLS */
    pub const AF_CAN: Self = Self(29); /* Controller Area Network      */
    pub const AF_TIPC: Self = Self(30); /* TIPC sockets			*/
    pub const AF_BLUETOOTH: Self = Self(31); /* Bluetooth sockets 		*/
    pub const AF_IUCV: Self = Self(32); /* IUCV sockets			*/
    pub const AF_RXRPC: Self = Self(33); /* RxRPC sockets 		*/
    pub const AF_ISDN: Self = Self(34); /* mISDN sockets 		*/
    pub const AF_PHONET: Self = Self(35); /* Phonet sockets		*/
    pub const AF_IEEE802154: Self = Self(36); /* IEEE802154 sockets		*/
    pub const AF_CAIF: Self = Self(37); /* CAIF sockets			*/
    pub const AF_ALG: Self = Self(38); /* Algorithm sockets		*/
    pub const AF_NFC: Self = Self(39); /* NFC sockets			*/
    pub const AF_VSOCK: Self = Self(40); /* vSockets			*/
    pub const AF_KCM: Self = Self(41); /* Kernel Connection Multiplexor*/
    pub const AF_QIPCRTR: Self = Self(42); /* Qualcomm IPC Router          */
    pub const AF_SMC: Self = Self(43); /* smc sockets: reserve number for
                                        * PF_SMC protocol family that
                                        * reuses AF_INET address family
                                        */
    pub const AF_XDP: Self = Self(44); /* XDP sockets			*/
    pub const AF_MCTP: Self = Self(45); /* Management component
                                         * transport protocol    */
    pub const AF_MAX: Self = Self(46); /* For now.. */
}

#[derive(Debug, Copy, Clone)]
pub struct SocketOptions(pub(crate) u32);

impl SocketOptions {
    #[inline]
    #[must_use]
    pub const fn new(socket_type: SocketType, socket_flags: SocketFlags) -> Self {
        Self(socket_type.0 | socket_flags.0)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct SocketType(pub(crate) u32);

impl SocketType {
    pub const SOCK_STREAM: Self = Self(1);
    pub const SOCK_DGRAM: Self = Self(2);
    pub const SOCK_RAW: Self = Self(3);
    pub const SOCK_RDM: Self = Self(4);
    pub const SOCK_SEQPACKET: Self = Self(5);
    /// Deprecated
    pub const SOCK_PACKET: Self = Self(10);
}

/// Defined in include/bits/socket_type.h actually an enum
transparent_bitflags!(
    pub struct SocketFlags: u32 {
        const DEFAULT = 0;
        const SOCK_NONBLOCK = linux_rust_bindings::fcntl::O_NONBLOCK as u32;
        const SOCK_CLOEXEC = linux_rust_bindings::fcntl::O_CLOEXEC as u32;
    }
);

#[derive(Debug, Copy, Clone)]
pub struct SocketArg {
    pub(crate) addr: SocketAddress,
    pub(crate) addr_len: usize,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct SocketAddress(linux_rust_bindings::socket::sockaddr_un);

impl SocketAddress {
    /// Get the `AddressFamily` of the socket address
    #[inline]
    #[must_use]
    pub fn family(&self) -> AddressFamily {
        AddressFamily(self.0.sun_family)
    }

    /// Get the raw path of the socket address
    #[inline]
    #[must_use]
    pub fn path_raw(&self) -> [core::ffi::c_char; 108] {
        self.0.sun_path
    }

    /// Tries to construct a `SocketAddress` from an `AsUnixStr` path
    /// # Errors
    /// The path is longer than 108 bytes (null termination included)
    pub fn try_from_unix<P: AsUnixStr>(path: P) -> crate::Result<SocketArg> {
        let mut ind = 0;
        let buf = path.exec_with_self_as_ptr(|ptr| unsafe {
            let mut buf = [0; 108];
            let mut ptr = ptr;
            while !ptr.is_null() {
                buf[ind] = ptr.read() as core::ffi::c_char;
                if ind == 107 && buf[ind] != 0 {
                    return Err(Error::no_code("Socket address too long"));
                } else if buf[ind] == 0 {
                    ind += 1;
                    break;
                }
                ptr = ptr.add(1);
                ind += 1;
            }
            Ok(buf)
        })?;
        let addr = Self(linux_rust_bindings::socket::sockaddr_un {
            sun_family: AddressFamily::AF_UNIX.0,
            sun_path: buf,
        });
        Ok(SocketArg {
            addr,
            addr_len: ind + 2,
        })
    }
}
