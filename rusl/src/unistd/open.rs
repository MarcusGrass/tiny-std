use sc::syscall;

use crate::platform::{Fd, Mode, OpenFlags, AT_FDCWD};
use crate::string::unix_str::UnixStr;

/// Attempts to open the fd at the path described by the null terminated pointer supplied.
/// # Errors
/// See `open`
/// # Safety
/// Pointer must be null terminated
#[inline]
pub unsafe fn open_raw(name_addr: usize, flags: OpenFlags) -> crate::Result<Fd> {
    let res = syscall!(OPENAT, AT_FDCWD, name_addr, flags.bits().0);
    Fd::coerce_from_register(res, "`OPENAT` syscall failed")
}

/// Attempts to open the fd at the path specified by a null terminated string, with the provided `OpenFlags`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// See above, errors are converted into an Err with the corresponding error code
pub fn open(path: &UnixStr, flags: OpenFlags) -> crate::Result<Fd> {
    let res = unsafe { syscall!(OPENAT, AT_FDCWD, path.as_ptr(), flags.bits().0) };
    Fd::coerce_from_register(res, "`OPENAT` syscall failed")
}

/// Attempts to open the fd at the path specified by a null terminated string, with the provided `OpenFlags` and `Mode`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// see above
#[inline]
pub fn open_mode(path: &UnixStr, flags: OpenFlags, mode: Mode) -> crate::Result<Fd> {
    let res = unsafe { syscall!(OPENAT, AT_FDCWD, path.as_ptr(), flags.bits().0, mode.bits()) };
    Fd::coerce_from_register(res, "`OPENAT` syscall failed")
}

/// Attempts to open a file at the specified path from the opened directory (`Fd`) with the specified `OpenFlags`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// see above
pub fn open_at(dir: Fd, path: &UnixStr, flags: OpenFlags) -> crate::Result<Fd> {
    let res = unsafe { syscall!(OPENAT, dir.0, path.as_ptr(), flags.bits().0) };
    Fd::coerce_from_register(res, "`OPENAT` syscall failed")
}

/// Attempts to open a file at the specified path from the opened directory (`Fd`) with the specified `OpenFlags` and `Mode`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// see above
pub fn open_at_mode(dir: Fd, path: &UnixStr, flags: OpenFlags, mode: Mode) -> crate::Result<Fd> {
    let res = unsafe { syscall!(OPENAT, dir.0, path.as_ptr(), flags.bits().0, mode.bits()) };
    Fd::coerce_from_register(res, "`OPENAT` syscall failed")
}

#[cfg(test)]
mod tests {
    use crate::string::unix_str::UnixStr;

    // Differences between tmp-file creating on `x86_64` and `aarch64`, pretty interesting
    // seems that we can't just name a dir on `aarch64` because it produces `EISDIR`
    #[test]
    fn try_open_temp() {
        let path = UnixStr::try_from_str("test-files\0").unwrap();
        try_open_temp_at_path(path).unwrap();
    }

    fn try_open_temp_at_path(path: &UnixStr) -> crate::Result<()> {
        use super::*;
        let _fd = open_mode(
            path,
            OpenFlags::O_WRONLY | OpenFlags::O_TMPFILE,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )?;
        let _fd = open_mode(
            path,
            OpenFlags::O_RDWR | OpenFlags::O_TMPFILE,
            Mode::S_IRGRP | Mode::S_IWGRP,
        )?;
        Ok(())
    }
}
