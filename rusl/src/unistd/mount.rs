use crate::platform::{FilesystemType, Mountflags};
use crate::string::unix_str::UnixStr;
use crate::Result;
use sc::syscall;

/// Mount a device.
/// Attempt to mount a device from `source` to `target` specifying a `FilesystemType` and `flags`.
/// Some filesystems allow providing additional data, which goes in `data`.
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/mount.2.html).
/// # Errors
/// See above
pub fn mount(
    source: &UnixStr,
    target: &UnixStr,
    fs_type: FilesystemType,
    flags: Mountflags,
    data: Option<&UnixStr>,
) -> Result<()> {
    unsafe {
        if let Some(data) = data {
            let res = syscall!(
                MOUNT,
                source.as_ptr(),
                target.as_ptr(),
                fs_type.0,
                flags.bits(),
                data.as_ptr()
            );
            bail_on_below_zero!(res, "`MOUNT` syscall failed");
        } else {
            let res = syscall!(
                MOUNT,
                source.as_ptr(),
                target.as_ptr(),
                fs_type.0,
                flags.bits(),
                0
            );
            bail_on_below_zero!(res, "`MOUNT` syscall failed");
        }
    }
    Ok(())
}

/// Unmount a device.
/// Attempts to unmount the device at `target`.
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/umount.2.html).
/// # Errors
/// See above.
pub fn unmount(target: &UnixStr) -> Result<()> {
    unsafe {
        let res = syscall!(UMOUNT2, target.as_ptr(), 0);
        bail_on_below_zero!(res, "`UNMOUNT2` syscall failed");
    }
    Ok(())
}
