pub use linux_rust_bindings::hidio::hiddev_devinfo;

// #define HIDIOCGDEVINFO		_IOR('H', 0x03, struct hiddev_devinfo)
pub const HIDIOCGDEV_INFO: u32 = crate::_ioc!(
    linux_rust_bindings::ioctl::_IOC_READ as u32,
    'H' as u32,
    0x03u32,
    core::mem::size_of::<linux_rust_bindings::hidio::hiddev_devinfo>() as u32,
    u32
);
