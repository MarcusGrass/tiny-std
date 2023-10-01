use core::mem::MaybeUninit;

use sc::syscall;

use crate::platform::{Fd, Stat, AT_FDCWD};
use crate::string::unix_str::UnixStr;

/// [stat](https://man7.org/linux/man-pages/man2/statx.2.html)
/// Gets file status at the path pointed to by `path`
/// # Errors
/// See above docs
#[inline]
pub fn stat(path: &UnixStr) -> crate::Result<Stat> {
    do_statat(AT_FDCWD, path)
}

/// [fstat](https://man7.org/linux/man-pages/man2/stat.2.html)
/// Gets file status at the relative offset from `dir_fd`
/// # Errors
/// See above docs
#[inline]
pub fn statat(dir_fd: Fd, path: &UnixStr) -> crate::Result<Stat> {
    do_statat(dir_fd.value(), path)
}

/// [fstat](https://man7.org/linux/man-pages/man2/stat.2.html)
/// Gets file status at the file pointed to by `Fd`
/// # Errors
/// See above docs
#[inline]
pub fn stat_fd(dir_fd: Fd) -> crate::Result<Stat> {
    do_statat(dir_fd.0, UnixStr::EMPTY)
}

fn do_statat(fd: i32, pathname: &UnixStr) -> crate::Result<Stat> {
    let mut stat = MaybeUninit::uninit();
    let res = unsafe {
        syscall!(
            NEWFSTATAT,
            fd,
            pathname.as_ptr(),
            stat.as_mut_ptr(),
            crate::platform::DirFlags::AT_EMPTY_PATH.bits().0
        )
    };
    bail_on_below_zero!(res, "`STAT` syscall failed");
    // Safety:
    // We're relying on the os to not supply a nullptr on success
    Ok(unsafe { stat.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_test() {
        let legit_path = UnixStr::try_from_str("test-files/can_stat.txt\0").unwrap();
        stat(legit_path).unwrap();
        stat(UnixStr::EMPTY).unwrap();
    }
}
