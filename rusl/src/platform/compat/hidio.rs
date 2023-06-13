// #define HIDIOCGDEVINFO		_IOR('H', 0x03, struct hiddev_devinfo)
pub const HIDIOCGDEV_INFO: u32 = crate::_ioc!(
    linux_rust_bindings::ioctl::_IOC_READ as u32,
    'H' as u32,
    0x03u32,
    comptime_hidio_size(),
    u32
);

#[allow(clippy::cast_possible_truncation)]
const fn comptime_hidio_size() -> u32 {
    let sz = core::mem::size_of::<linux_rust_bindings::hidio::hiddev_devinfo>();
    assert!(
        sz < u32::MAX as usize,
        "The size of hiddev_devinfo is larger than an u32"
    );
    sz as u32
}
