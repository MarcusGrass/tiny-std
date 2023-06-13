use linux_rust_bindings::ioctl::{_IOC_NONE, _IOC_READ, _IOC_WRITE};
pub use linux_rust_bindings::usb::{usb_device_descriptor, usbdevfs_bulktransfer};

#[allow(clippy::cast_possible_truncation)]
pub const USBDEVFS_BULK: u32 = crate::_ioc!(
    _IOC_READ as u32 | _IOC_WRITE as u32,
    'U' as u32,
    2u32,
    core::mem::size_of::<usbdevfs_bulktransfer>() as u32,
    u32
);

pub const USBDEVFS_CLAIM_INTERFACE: u32 = crate::_ior!('U' as u32, 15u32, u32);
pub const USBDEVFS_RELEASE_INTERFACE: u32 = crate::_ior!('U' as u32, 16u32, u32);
pub const USBDEVFS_RESET: u32 = crate::_ioc!(_IOC_NONE as u32, 'U' as u32, 20u32, 0u32, u32);
