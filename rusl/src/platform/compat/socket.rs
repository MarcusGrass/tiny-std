//! These are not defined in the uapi which is a bit hairy, if they change, that's obviously
use crate::platform::Fd;
use crate::string::unix_str::UnixStr;
/// a problem.
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

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct SocketAddressInet(pub(crate) linux_rust_bindings::socket::sockaddr_in);

impl SocketAddressInet {
    pub const LENGTH: usize = core::mem::size_of::<linux_rust_bindings::socket::sockaddr_in>();

    #[must_use]
    pub const fn new(ip_addr: [u8; 4], port: u16) -> Self {
        let s_addr = u32::from_be_bytes(ip_addr);
        let port_bytes = port.to_ne_bytes();
        let sin_port = u16::from_be_bytes(port_bytes);
        let i = linux_rust_bindings::socket::sockaddr_in {
            sin_family: AddressFamily::AF_INET.0,
            sin_port,
            sin_addr: linux_rust_bindings::socket::in_addr { s_addr },
            __pad: [0u8; 8],
        };
        Self(i)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SocketArgUnix {
    pub(crate) addr: SocketAddressUnix,
    pub(crate) addr_len: usize,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct SocketAddressUnix(pub(crate) linux_rust_bindings::socket::sockaddr_un);

impl SocketAddressUnix {
    /// Get the `AddressFamily` of the socket address
    #[inline]
    #[must_use]
    pub const fn family(&self) -> AddressFamily {
        AddressFamily(self.0.sun_family)
    }

    /// Get the raw path of the socket address
    #[inline]
    #[must_use]
    pub const fn path_raw(&self) -> [core::ffi::c_char; 108] {
        self.0.sun_path
    }

    /// Tries to construct a `SocketAddress` from a `UnixStr` path
    /// # Errors
    /// The path is longer than 108 bytes (null termination included).
    /// The path contains byte values out of the 7-bit ASCII range.
    pub fn try_from_unix(path: &UnixStr) -> crate::Result<SocketArgUnix> {
        let mut ind = 0;
        let buf = unsafe {
            let mut buf = [0; 108];
            let mut ptr = path.as_ptr();
            while !ptr.is_null() {
                let val = core::ffi::c_char::try_from(ptr.read()).map_err(|_e| {
                    Error::no_code(
                        "Socket paths need to be 7-bit ASCII, path contained value in 8-bit range",
                    )
                })?;
                buf[ind] = val;
                if ind == 107 && buf[ind] != 0 {
                    return Err(Error::no_code("Socket address too long"));
                } else if buf[ind] == 0 {
                    ind += 1;
                    break;
                }
                ptr = ptr.add(1);
                ind += 1;
            }
            buf
        };
        let addr = Self(linux_rust_bindings::socket::sockaddr_un {
            sun_family: AddressFamily::AF_UNIX.0,
            sun_path: buf,
        });
        Ok(SocketArgUnix {
            addr,
            addr_len: ind
                + core::mem::size_of::<linux_rust_bindings::socket::__kernel_sa_family_t>(),
        })
    }
}

//  #define CMSG_LEN(len)   (CMSG_ALIGN (sizeof (struct cmsghdr)) + (len))
#[cfg(feature = "alloc")]
macro_rules! cmsg_len {
    ($len: expr) => {
        cmsg_align!(core::mem::size_of::<CmsgHdr>()) + $len
    };
}

//  #define CMSG_SPACE(len) (CMSG_ALIGN (len) + CMSG_ALIGN (sizeof (struct cmsghdr)))
#[cfg(feature = "alloc")]
macro_rules! cmsg_space {
    ($len: expr) => {
        cmsg_align!($len) + cmsg_align!(core::mem::size_of::<CmsgHdr>())
    };
}

// #define CMSG_ALIGN(len) (((len) + sizeof (size_t) - 1) & (size_t) ~(sizeof (size_t) - 1))
#[cfg(feature = "alloc")]
macro_rules! cmsg_align {
    ($len: expr) => {
        ($len + core::mem::size_of::<usize>() - 1) & !(core::mem::size_of::<usize>() - 1)
    };
}

// #define CMSG_FIRSTHDR(mhdr) ((size_t) (mhdr)->msg_controllen >= sizeof (struct cmsghdr) ? (struct cmsghdr *) (mhdr)->msg_control : (struct cmsghdr *) 0)
#[cfg(feature = "alloc")]
macro_rules! cmsg_firsthdr {
    ($mhdr: expr) => {
        if $mhdr.msg_controllen >= core::mem::size_of::<CmsgHdr>() {
            $mhdr.msg_control
        } else {
            core::ptr::null_mut()
        }
    };
}

/*
#define CMSG_NXTHDR(mhdr, cmsg) ((cmsg)->cmsg_len < sizeof (struct cmsghdr) || \
    __CMSG_LEN(cmsg) + sizeof(struct cmsghdr) >= __MHDR_END(mhdr) - (unsigned char *)(cmsg) \
    ? 0 : (struct cmsghdr *)__CMSG_NEXT(cmsg))
 */
#[cfg(feature = "alloc")]
macro_rules! cmsg_nxthdr {
    ($mhdr: expr, $cmsg: expr) => {
        if ((*$cmsg).cmsg_len) < core::mem::size_of::<CmsgHdr>()
            || __cmsg_len!($cmsg) + core::mem::size_of::<CmsgHdr>()
                >= __mhdr_end!($mhdr) - core::ptr::addr_of!($cmsg) as usize
        {
            core::ptr::null()
        } else {
            __cmsg_next!($cmsg) as *const CmsgHdr
        }
    };
}

//  #define CMSG_DATA(cmsg) ((unsigned char *) (((struct cmsghdr *)(cmsg)) + 1))
#[cfg(feature = "alloc")]
macro_rules! cmsg_data {
    ($cmsg: expr) => {
        ($cmsg.cast::<CmsgHdr>()).add(1).cast::<u8>()
    };
}

// #define __CMSG_LEN(cmsg) (((cmsg)->cmsg_len + sizeof(long) - 1) & ~(long)(sizeof(long) - 1))
#[cfg(feature = "alloc")]
macro_rules! __cmsg_len {
    ($cmsg: expr) => {
        ((*$cmsg).cmsg_len + 8usize - 1usize) & !((8u64 - 1u64) as usize)
    };
}

// #define __CMSG_NEXT(cmsg) ((unsigned char *)(cmsg) + __CMSG_LEN(cmsg))
#[cfg(feature = "alloc")]
macro_rules! __cmsg_next {
    ($cmsg: expr) => {
        $cmsg as usize + __cmsg_len!($cmsg)
    };
}
// #define __MHDR_END(mhdr) ((unsigned char *)(mhdr)->msg_control + (mhdr)->msg_controllen)
#[cfg(feature = "alloc")]
macro_rules! __mhdr_end {
    ($mhdr: expr) => {
        core::ptr::addr_of!($mhdr.msg_control) as usize + $mhdr.msg_controllen
    };
}

#[derive(Debug)]
#[cfg(feature = "alloc")]
pub enum ControlMessageSend<'a> {
    ScmRights(&'a [crate::platform::Fd]),
}

#[repr(C)]
#[derive(Debug)]
#[cfg(feature = "alloc")]
pub struct MsgHdrBorrow<'a> {
    msg_name: *const u8,
    msg_namelen: u32,
    msg_iov: *mut linux_rust_bindings::uio::iovec,
    msg_iovlen: usize,
    msg_control: *mut CmsgHdrSend<'a>,
    msg_controllen: usize,
    msg_flags: i32,
}

#[cfg(feature = "alloc")]
pub struct SendDropGuard<'a> {
    pub(crate) msghdr: MsgHdrBorrow<'a>,
    _dealloc_spc: alloc::vec::Vec<u8>,
}
#[cfg(feature = "alloc")]
#[allow(clippy::cast_possible_truncation)]
impl<'a> MsgHdrBorrow<'a> {
    #[must_use]
    pub fn create_send(
        name: Option<&'a UnixStr>,
        io: &'a [crate::platform::IoSlice<'a>],
        control: Option<ControlMessageSend<'a>>,
    ) -> SendDropGuard<'a> {
        let (name, name_len) = if let Some(name) = name {
            (name.as_ptr(), name.len() as u32)
        } else {
            (core::ptr::null(), 0)
        };
        unsafe {
            if let Some(ctrl) = control {
                match ctrl {
                    ControlMessageSend::ScmRights(fds) => {
                        let spc = cmsg_space!(core::mem::size_of_val(fds));
                        let mut cmsg_raw = alloc::vec![0u8; spc];
                        let cmsg_ptr = cmsg_raw.as_mut_ptr();
                        let mhdr = MsgHdrBorrow {
                            msg_name: name,
                            msg_namelen: name_len,
                            msg_iov: io.as_ptr().cast_mut().cast(),
                            msg_iovlen: io.len(),
                            msg_control: cmsg_ptr.cast(),
                            msg_controllen: spc,
                            msg_flags: 0,
                        };
                        let cmhdr: *mut CmsgHdr = cmsg_firsthdr!(mhdr).cast::<CmsgHdr>();
                        // Space was just created for this.
                        let mut_cm = cmhdr.as_mut().unwrap_unchecked();
                        let cmsg_len = cmsg_len!(core::mem::size_of_val(fds));
                        mut_cm.cmsg_level = 1;
                        mut_cm.cmsg_type = 0x01;
                        mut_cm.cmsg_len = cmsg_len;
                        let data = cmsg_data!(cmhdr);
                        core::ptr::copy_nonoverlapping(
                            fds.as_ptr().cast::<u8>(),
                            data,
                            core::mem::size_of_val(fds),
                        );
                        SendDropGuard {
                            msghdr: mhdr,
                            _dealloc_spc: cmsg_raw,
                        }
                    }
                }
            } else {
                SendDropGuard {
                    msghdr: MsgHdrBorrow {
                        msg_name: name,
                        msg_namelen: name_len,
                        msg_iov: io.as_ptr().cast_mut().cast(),
                        msg_iovlen: io.len(),
                        msg_control: core::ptr::null_mut(),
                        msg_controllen: 0,
                        msg_flags: 0,
                    },
                    _dealloc_spc: alloc::vec::Vec::new(),
                }
            }
        }
    }

    #[must_use]
    pub fn create_recv(
        io: &'a mut [crate::platform::IoSliceMut<'a>],
        cmsg_buf: Option<&'a mut [u8]>,
    ) -> Self {
        let (ctrl, ctrl_len) = if let Some(cm_buf) = cmsg_buf {
            (cm_buf.as_mut_ptr(), cm_buf.len())
        } else {
            (core::ptr::null_mut(), 0)
        };
        MsgHdrBorrow {
            msg_name: core::ptr::null(),
            msg_namelen: 0,
            msg_iov: io.as_mut_ptr().cast(),
            msg_iovlen: io.len(),
            msg_control: ctrl.cast(),
            msg_controllen: ctrl_len,
            msg_flags: 0,
        }
    }

    #[must_use]
    pub fn control_messages(&'a self) -> ControlMessageIterator<'a> {
        let first = cmsg_firsthdr!(self);
        let cmsg_prev = if first.is_null() {
            None
        } else {
            Some(first.cast())
        };
        ControlMessageIterator {
            msghdr: self,
            cmsg_prev,
        }
    }
}

#[cfg(feature = "alloc")]
pub struct ControlMessageIterator<'a> {
    msghdr: &'a MsgHdrBorrow<'a>,
    cmsg_prev: Option<*mut CmsgHdr>,
}

#[cfg(feature = "alloc")]
impl<'a> Iterator for ControlMessageIterator<'a> {
    type Item = ControlMessageSend<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cmsg = self.cmsg_prev?;
        unsafe {
            let r = cmsg.as_mut()?;
            if r.cmsg_type == 1 && r.cmsg_level == 1 {
                let data = cmsg_data!(cmsg);
                let len = cmsg.cast_const() as usize + r.cmsg_len - data as usize;
                let len = len / core::mem::size_of::<crate::platform::Fd>();
                self.cmsg_prev = Some(cmsg_nxthdr!(self.msghdr, cmsg).cast_mut());
                // Here on x86_64 and aarch64 data is 8-byte aligned
                #[allow(clippy::cast_ptr_alignment)]
                return Some(ControlMessageSend::ScmRights(core::slice::from_raw_parts(
                    data as *const crate::platform::NonNegativeI32,
                    len,
                )));
            }
            self.cmsg_prev = Some(cmsg_nxthdr!(self.msghdr, cmsg).cast_mut());
            self.next()
        }
    }
}

/// A control message, metadata about the message type, followed by bytes of data,
/// this struct can be thought of as a protocol for an array of bytes, the control-message itself is functionally a variable length array of bytes, this should never go on the stack, since
/// the actual payload would go outside of the bounds of this struct. Allocating bytes [`[u8; N]`] on the stack, then constructing a cmsg-pointer to that is alright though, use with caution.
#[repr(C)]
#[derive(Debug)]
#[cfg(feature = "alloc")]
pub struct CmsgHdrSend<'a> {
    cmsg_len: usize,
    cmsg_level: i32,
    cmsg_type: i32,
    /// Hope this is actually not compiled into to the struct...
    _pd: core::marker::PhantomData<&'a ()>,
    // Data is written to the end here, it's not a pointer, it's variable size data directly in the struct, this can never be put on the stack.
}

#[repr(C)]
#[cfg(feature = "alloc")]
pub struct MsgHdr {
    pub msg_name: *const u8,
    pub msg_namelen: u32,
    pub msg_iov: *mut linux_rust_bindings::uio::iovec,
    pub msg_iovlen: usize,
    pub msg_control: *mut CmsgHdr,
    pub msg_controllen: usize,
    pub msg_flags: i32,
}

#[derive(Debug)]
pub enum ControlMessageRaw {
    ScmRights(*mut Fd, usize),
}

#[cfg(feature = "alloc")]
impl MsgHdr {
    /// Update control message
    /// # Safety
    /// `cmsg_ptr` is valid and has enough space to write the cmsg
    /// `cmsg_ptr` needs to be alive as long as this struct is alive.
    #[cfg(feature = "alloc")]
    pub unsafe fn update_control(
        &mut self,
        control: Option<ControlMessageSend>,
        cmsg_ptr: *mut u8,
    ) {
        unsafe {
            if let Some(ctrl) = control {
                match ctrl {
                    ControlMessageSend::ScmRights(fds) => {
                        let spc = cmsg_space!(core::mem::size_of_val(fds));
                        core::ptr::write_bytes(cmsg_ptr, 0, spc);
                        self.msg_control = cmsg_ptr.cast();
                        self.msg_controllen = spc;
                        let cmhdr: *mut CmsgHdr = cmsg_firsthdr!(self).cast::<CmsgHdr>();
                        // Space was just created for this.
                        let mut_cm = cmhdr.as_mut().unwrap_unchecked();
                        let cmsg_len = cmsg_len!(core::mem::size_of_val(fds));
                        mut_cm.cmsg_level = 1;
                        mut_cm.cmsg_type = 0x01;
                        mut_cm.cmsg_len = cmsg_len;
                        let data = cmsg_data!(cmhdr);
                        core::ptr::copy_nonoverlapping(
                            fds.as_ptr().cast::<u8>(),
                            data,
                            core::mem::size_of_val(fds),
                        );
                    }
                }
            } else {
                self.msg_control = core::ptr::null_mut();
                self.msg_controllen = 0;
            }
        }
    }

    /// Create a [`MsgHdr`] ready to send using a sendmsg
    /// # Safety
    /// All pointers must be valid.
    /// All pointers must live at least as long as this struct.
    ///
    #[must_use]
    #[cfg(feature = "alloc")]
    pub unsafe fn create_send(
        msg_iov: *mut linux_rust_bindings::uio::iovec,
        msg_iovlen: usize,
        control: Option<ControlMessageRaw>,
        cmsg_ptr: *mut u8,
    ) -> Self {
        unsafe {
            if let Some(ctrl) = control {
                match ctrl {
                    ControlMessageRaw::ScmRights(fds, len) => {
                        let fd_buf = core::slice::from_raw_parts_mut(fds, len);
                        let spc = cmsg_space!(core::mem::size_of_val(fd_buf));
                        core::ptr::write_bytes(cmsg_ptr, 0, spc);
                        let mhdr = Self {
                            msg_name: core::ptr::null(),
                            msg_namelen: 0,
                            msg_iov,
                            msg_iovlen: msg_iovlen as _,
                            msg_control: cmsg_ptr.cast(),
                            msg_controllen: spc,
                            msg_flags: 0,
                        };
                        let cmhdr: *mut CmsgHdr = cmsg_firsthdr!(mhdr).cast::<CmsgHdr>();
                        // Space was just created for this.
                        let mut_cm = cmhdr.as_mut().unwrap_unchecked();
                        let cmsg_len = cmsg_len!(core::mem::size_of_val(fd_buf));
                        mut_cm.cmsg_level = 1;
                        mut_cm.cmsg_type = 0x01;
                        mut_cm.cmsg_len = cmsg_len;
                        let data = cmsg_data!(cmhdr);
                        core::ptr::copy_nonoverlapping(
                            fds.cast_const().cast::<u8>(),
                            data,
                            core::mem::size_of_val(fd_buf),
                        );
                        mhdr
                    }
                }
            } else {
                Self {
                    msg_name: core::ptr::null(),
                    msg_namelen: 0,
                    msg_iov,
                    msg_iovlen: msg_iovlen as _,
                    msg_control: core::ptr::null_mut(),
                    msg_controllen: 0,
                    msg_flags: 0,
                }
            }
        }
    }
}

/// A control message, metadata about the message type, followed by bytes of data,
/// this struct can be thought of as a protocol for an array of bytes, the control-message itself is functionally a variable length array of bytes, this should never go on the stack, since
/// the actual payload would go outside of the bounds of this struct. Allocating bytes [`[u8; N]`] on the stack, then constructing a cmsg-pointer to that is alright though, use with caution.
#[repr(C)]
#[derive(Debug)]
#[cfg(feature = "alloc")]
pub struct CmsgHdr {
    cmsg_len: usize,
    cmsg_level: i32,
    cmsg_type: i32,
    // Data is written to the end here, it's not a pointer, it's variable size data directly in the struct, this can never be put on the stack.
}

#[cfg(test)]
#[cfg(feature = "alloc")]
#[allow(
    clippy::cast_ptr_alignment,
    clippy::ptr_cast_constness,
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation
)]
mod tests {
    use crate::platform::{CmsgHdr, Fd, IoSlice, MsgHdr, OpenFlags};
    use crate::string::unix_str::UnixStr;
    use crate::unistd::open;
    use alloc::vec;
    use core::ptr::null_mut;
    use linux_rust_bindings::uio::iovec;

    #[test]
    fn cmsg_macros_on_empty() {
        unsafe {
            let mut cmsg = CmsgHdr {
                cmsg_len: 16,
                cmsg_level: 0,
                cmsg_type: 0,
            };
            let cmsg_ptr = core::ptr::addr_of_mut!(cmsg);
            let len = __cmsg_len!(cmsg_ptr);
            assert_eq!(16, len);
            let _ = __cmsg_next!(cmsg_ptr);
            let mhdr = MsgHdr {
                msg_name: "abc".as_ptr(),
                msg_namelen: 0,
                msg_iov: null_mut(),
                msg_iovlen: 0,
                msg_control: cmsg_ptr,
                msg_controllen: 0,
                msg_flags: 0,
            };
            let _ = __mhdr_end!(mhdr);
            assert!(cmsg_firsthdr!(mhdr).is_null());
        }
    }

    /// pretty unsafe, references need to live until this [`MsgHdr`] goes out of scope, cba for this test though
    /// Also leaks the cap vector
    unsafe fn msg_hdr(name: &UnixStr, iovec: &[iovec], fds: &[Fd]) -> MsgHdr {
        let spc = cmsg_space!(core::mem::size_of_val(fds));
        let mut cap = vec![0u8; spc];
        let cmsg_ptr = cap.as_mut_ptr();
        let mhdr = MsgHdr {
            msg_name: name.as_ptr(),
            msg_namelen: name.len() as _,
            msg_iov: iovec.as_ptr().cast_mut(),
            msg_iovlen: iovec.len(),
            msg_control: cmsg_ptr.cast(),
            msg_controllen: spc,
            msg_flags: 0,
        };
        let cmhdr: *mut CmsgHdr = cmsg_firsthdr!(mhdr);
        assert!(!cmhdr.is_null());
        let mut_cm = cmhdr.as_mut().unwrap();
        let cmsg_len = cmsg_len!(core::mem::size_of_val(fds));
        mut_cm.cmsg_level = 1;
        mut_cm.cmsg_type = 0x01;
        mut_cm.cmsg_len = cmsg_len;
        let data = cmsg_data!(cmhdr);
        core::ptr::copy_nonoverlapping(
            core::ptr::from_ref(fds) as *const u8,
            data,
            core::mem::size_of_val(fds),
        );
        core::mem::forget(cap);
        mhdr
    }

    #[test]
    fn cmsg_macros_has_single_fd() {
        let name = unix_lit!("name");
        let io = IoSlice::new(b"hello");
        let fd1 = open(unix_lit!("/proc/mounts"), OpenFlags::O_RDONLY).unwrap();
        let v = &[io.vec];
        let f = &[fd1];
        let mhdr = unsafe { msg_hdr(name, v, f) };
        let first_hdr = cmsg_firsthdr!(mhdr);
        assert!(!first_hdr.is_null());
        let first_hdr_deref = unsafe { first_hdr.as_ref().unwrap() };
        assert_eq!(1, first_hdr_deref.cmsg_level);
        assert_eq!(1, first_hdr_deref.cmsg_type);
        // 8 bytes length, 4 bytes level, 4 bytes type, 4 bytes [`Fd`] 4 bytes alignment padding
        assert_eq!(20, first_hdr_deref.cmsg_len);
        unsafe {
            let data = cmsg_data!(first_hdr);
            let _slice = core::slice::from_raw_parts(data as _, 8);
            // From the back
            let len = first_hdr as *const _ as usize + first_hdr_deref.cmsg_len - data as usize;
            let _gross_count = len / core::mem::size_of::<Fd>();
            let null_term_ptr = data as *mut Fd;
            let first = null_term_ptr.read_unaligned();
            assert_eq!(fd1, first);
            assert_eq!(0, cmsg_nxthdr!(mhdr, first_hdr) as usize);
        }
    }

    #[test]
    fn cmsg_macros_has_two_fd() {
        let name = unix_lit!("name");
        let io = IoSlice::new(b"hello");
        let fd1 = open(unix_lit!("/proc/mounts"), OpenFlags::O_RDONLY).unwrap();
        let v = &[io.vec];
        let f = &[fd1];
        let mhdr = unsafe { msg_hdr(name, v, f) };
        let first_hdr = cmsg_firsthdr!(mhdr);
        assert!(!first_hdr.is_null());
        let first_hdr_deref = unsafe { first_hdr.as_ref().unwrap() };
        assert_eq!(1, first_hdr_deref.cmsg_level);
        assert_eq!(1, first_hdr_deref.cmsg_type);
        // 8 bytes length, 4 bytes level, 4 bytes type, 4 bytes [`Fd`] 4 bytes alignment padding
        assert_eq!(20, first_hdr_deref.cmsg_len);
        unsafe {
            let data = cmsg_data!(first_hdr);
            let _slice = core::slice::from_raw_parts(data as _, 8);
            // From the back
            let len = first_hdr as *const _ as usize + first_hdr_deref.cmsg_len - data as usize;
            let _gross_count = len / core::mem::size_of::<Fd>();
            let null_term_ptr = data as *mut Fd;
            let first = null_term_ptr.read_unaligned();
            assert_eq!(fd1, first);
            assert_eq!(0, cmsg_nxthdr!(mhdr, first_hdr) as usize);
        }
    }
}
