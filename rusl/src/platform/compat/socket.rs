//! These are not defined in the uapi which is a bit hairy, if they change, that's obviously
/// a problem.
use crate::string::unix_str::AsUnixStr;
use crate::Error;

transparent_bitflags! {
    pub struct AddressFamily: u16 {
        const AF_UNSPEC =	0;
        const AF_UNIX =		1;	/* Unix domain sockets 		*/
        const AF_LOCAL =	1;	/* POSIX name for AF_UNIX	*/
        const AF_INET =		2;	/* Internet IP Protocol 	*/
        const AF_AX25 =		3;	/* Amateur Radio AX.25 		*/
        const AF_IPX =		4;	/* Novell IPX 			*/
        const AF_APPLETALK =	5;	/* AppleTalk DDP 		*/
        const AF_NETROM =	6;	/* Amateur Radio NET/ROM 	*/
        const AF_BRIDGE =	7;	/* Multiprotocol bridge 	*/
        const AF_ATMPVC =	8;	/* ATM PVCs			*/
        const AF_X25 =		9;	/* Reserved for X.25 project 	*/
        const AF_INET6 =	10;	/* IP version 6			*/
        const AF_ROSE =		11;	/* Amateur Radio X.25 PLP	*/
        #[allow(non_upper_case_globals)]
        const AF_DECnet =	12;	/* Reserved for DECnet project	*/
        const AF_NETBEUI =	13;	/* Reserved for 802.2LLC project*/
        const AF_SECURITY =	14;	/* Security callback pseudo AF */
        const AF_KEY =		15;      /* PF_KEY key management API */
        const AF_NETLINK =	16;
        const AF_ROUTE =	Self::AF_NETLINK.bits(); /* Alias to emulate 4.4BSD */
        const AF_PACKET =	17;	/* Packet family		*/
        const AF_ASH =		18;	/* Ash				*/
        const AF_ECONET =	19;	/* Acorn Econet			*/
        const AF_ATMSVC =	20;	/* ATM SVCs			*/
        const AF_RDS =		21;	/* RDS sockets 			*/
        const AF_SNA =		22;	/* Linux SNA Project (nutters!) */
        const AF_IRDA =		23;	/* IRDA sockets			*/
        const AF_PPPOX =	24;	/* PPPoX sockets		*/
        const AF_WANPIPE =	25;	/* Wanpipe API Sockets */
        const AF_LLC =		26;	/* Linux LLC			*/
        const AF_IB =		27;	/* Native InfiniBand address	*/
        const AF_MPLS =		28;	/* MPLS */
        const AF_CAN =		29;	/* Controller Area Network      */
        const AF_TIPC =		30;	/* TIPC sockets			*/
        const AF_BLUETOOTH =	31;	/* Bluetooth sockets 		*/
        const AF_IUCV =		32;	/* IUCV sockets			*/
        const AF_RXRPC =	33;	/* RxRPC sockets 		*/
        const AF_ISDN =		34;	/* mISDN sockets 		*/
        const AF_PHONET =	35;	/* Phonet sockets		*/
        const AF_IEEE802154 =	36;	/* IEEE802154 sockets		*/
        const AF_CAIF =		37;	/* CAIF sockets			*/
        const AF_ALG =		38;	/* Algorithm sockets		*/
        const AF_NFC =		39;	/* NFC sockets			*/
        const AF_VSOCK =	40;	/* vSockets			*/
        const AF_KCM =		41;	/* Kernel Connection Multiplexor*/
        const AF_QIPCRTR =	42;	/* Qualcomm IPC Router          */
        const AF_SMC =		43;	/* smc sockets: reserve number for
                                 * PF_SMC protocol family that
                                 * reuses AF_INET address family
                                 */
        const AF_XDP =		44;	/* XDP sockets			*/
        const AF_MCTP =		45;	/* Management component
                                * transport protocol    */
        const AF_MAX =		46;	/* For now.. */
    }
}

/// Defined in include/bits/socket_type.h actually an enum
transparent_bitflags!(
    pub struct SocketType: i32 {
        const SOCK_STREAM = 1;
        const SOCK_DGRAM = 2;
        const SOCK_RAW = 3;
        const SOCK_RDM = 4;
        const SOCK_SEQPACKET = 5;
        /// Deprecated
        const SOCK_PACKET = 10;
        const SOCK_NONBLOCK = linux_rust_bindings::fcntl::O_NONBLOCK;
        const SOCK_CLOEXEC = linux_rust_bindings::fcntl::O_CLOEXEC;
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
        self.0.sun_family.into()
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
            sun_family: AddressFamily::AF_UNIX.bits(),
            sun_path: buf,
        });
        Ok(SocketArg {
            addr,
            addr_len: ind + 2,
        })
    }
}
