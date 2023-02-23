use linux_rust_bindings::usb::usbdevfs_bulktransfer;

use crate::ioctl::ioctl;
use crate::platform::{Fd, USBDEVFS_BULK, USBDEVFS_CLAIM_INTERFACE, USBDEVFS_RELEASE_INTERFACE};
use crate::Result;

/// Make a bulk transfer on the hid fd specified by `fd` at `endpoint`, `data` is either an
/// input or output buffer, `timeout` is specified in milliseconds.
/// # Errors
/// Various `ioctl` errors, like a bad fd, bad endpoint, buffer too small etc.
pub fn bulk_transfer(fd: Fd, endpoint: u32, data: &mut [u8], timeout: u32) -> Result<usize> {
    let mut bulk = usbdevfs_bulktransfer {
        ep: endpoint,
        len: data.len() as u32,
        timeout,
        data: data.as_mut_ptr().cast(),
    };
    unsafe {
        ioctl(
            fd as usize,
            USBDEVFS_BULK as usize,
            core::ptr::addr_of_mut!(bulk) as usize,
        )
    }
}

pub fn claim_interface(fd: Fd, interface_number: u32) -> Result<()> {
    unsafe {
        ioctl(
            fd as usize,
            USBDEVFS_CLAIM_INTERFACE as usize,
            core::ptr::addr_of!(interface_number) as usize,
        )?;
    }
    Ok(())
}

pub fn release_interface(fd: Fd, interface_number: u32) -> Result<()> {
    unsafe {
        ioctl(
            fd as usize,
            USBDEVFS_RELEASE_INTERFACE as usize,
            interface_number as usize,
        )?;
    }
    Ok(())
}
