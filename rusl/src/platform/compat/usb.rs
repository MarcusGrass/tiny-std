use linux_rust_bindings::ioctl::{_IOC_READ, _IOC_WRITE};
pub use linux_rust_bindings::usb::{usb_device_descriptor, usbdevfs_bulktransfer};

pub const USBDEVFS_BULK: u32 = crate::_ioc!(
    _IOC_READ as u32 | _IOC_WRITE as u32,
    'U' as u32,
    2u32,
    core::mem::size_of::<usbdevfs_bulktransfer>() as u32,
    u32
);
