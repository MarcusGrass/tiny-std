use core::mem::MaybeUninit;

use linux_rust_bindings::hidio::hiddev_devinfo;

use crate::ioctl::ioctl;
use crate::platform::{Fd, HIDIOCGDEV_INFO};
use crate::Result;

/// Get `hiddev_devinfo` of the device connected to the provided `fd`.  
/// # Errors
/// Various `ioctl` errors.  
pub fn get_hid_dev_dev_info(fd: Fd) -> Result<hiddev_devinfo> {
    let mut dev_info_uninit = MaybeUninit::uninit();
    unsafe {
        ioctl(
            fd as usize,
            HIDIOCGDEV_INFO as usize,
            dev_info_uninit.as_mut_ptr() as usize,
        )?;
        Ok(dev_info_uninit.assume_init())
    }
}
