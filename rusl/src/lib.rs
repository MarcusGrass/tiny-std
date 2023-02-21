#![allow(unused_doc_comments)]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use error::{Error, Result};

pub mod error;
#[macro_use]
pub(crate) mod macros;
pub mod hidio;
pub mod io_uring;
pub mod ioctl;
pub mod network;
pub mod platform;
pub mod process;
pub mod select;
pub mod string;
pub mod termios;
pub mod time;
pub mod unistd;
pub mod usb;

#[cfg(test)]
mod tests {
    use crate::ioctl::ioctl;
    use crate::platform::OpenFlags;
    use crate::unistd::open;

    #[test]
    fn test_yubi() {
        let fd = open("/dev/hidraw0", OpenFlags::O_RDWR).unwrap();
        let res = unsafe { ioctl(fd as usize, 0x03, 0x03).unwrap() };
    }
}
