pub use accept::accept;
pub use bind::bind;
pub use connect::connect;
pub use listen::listen;
pub use socket::{socket, Domain, SocketType};

use crate::compat::unix_str::AsUnixStr;
use crate::{Error, Result};

mod accept;
mod bind;
mod connect;
mod listen;
mod socket;

pub struct SocketArg {
    addr: SocketAddress,
    addr_len: usize,
}

#[repr(C)]
pub struct SocketAddress {
    pub sun_family: u16,
    pub sun_path: [u8; 108],
}

impl SocketAddress {
    /// Tries to construct a `SocketAddress` from an `AsUnixStr` path
    /// # Errors
    /// The path is longer than 108 bytes (null termination included)
    pub fn try_from_unix<P: AsUnixStr>(path: P, family: Domain) -> Result<SocketArg> {
        let mut ind = 0;
        let buf = path.exec_with_self_as_ptr(|ptr| unsafe {
            let mut buf = [0u8; 108];
            let mut ptr = ptr;
            while !ptr.is_null() {
                buf[ind] = ptr.read();
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
        let addr = Self {
            sun_family: family.bits() as u16,
            sun_path: buf,
        };
        Ok(SocketArg {
            addr,
            addr_len: ind + 2,
        })
    }
}
