use sc::syscall;

use crate::compat::unix_str::AsUnixStr;
use crate::platform::{Fd, AT_FDCWD};

/// Openflags accepted by the [open syscall](https://man7.org/linux/man-pages/man2/open.2.html)
transparent_bitflags! {
    pub struct OpenFlags: i32 {
        const O_RDONLY = 0;
        const O_WRONLY = 1;
        const O_RDWR = 2;
        const O_APPEND = 1024;
        const O_ASYNC = 0x2000;
        const O_CLOEXEC = 0x80000;
        const O_CREAT = 64;
        const O_DIRECT = 0x4000;
        const O_DIRECTORY = 0x10000;
        const O_DSYNC = 4096;
        const O_EXCL = 128;
        const O_LARGEFILE = 0;
        const O_NOATIME = 0o1000000;
        const O_NOCTTY = 256;
        const O_NOFOLLOW = 0x20000;
        const O_NONBLOCK = 2048;
        const O_NDELAY = 0x800;
        const O_PATH = 0o10000000;
        const O_SYNC = 1052672;
        const O_TMPFILE = 0o20000000 | 0x10000;
        const O_TRUNC = 512;
    }
}

/// Mode accepted by the [open syscall](https://man7.org/linux/man-pages/man2/open.2.html)
transparent_bitflags! {
    pub struct Mode: u32 {
        const S_IRWXU = 0o0000700; // 00700 user read write exec
        const S_IRUSR = 0o0000400; // 00400 user Read
        const S_IWUSR = 0o0000200; // 00200 user write
        const S_IXUSR = 0o0000100;  // 00100 user execute
        const S_IRWXG = 0o0000070;  // 00070 group read write exec
        const S_IRGRP = 0o0000040;  // 00040 group read
        const S_IWGRP = 0o0000020;  // 00020 group write
        const S_IXGRP = 0o0000010;   // 00010 group exec
        const S_IRWXO = 0o0000007;   // 00007 other read write exec
        const S_IROTH = 0o0000004;   // 00004 other read
        const S_IWOTH = 0o0000002;   // 00002 other write
        const S_IXOTH = 0o0000001;   // 00001 other execute

        // Linux specific bits
        const S_ISUID = 0o0004000; // 0004000 set-user-ID bit
        const S_ISGID = 0o0002000; // 0002000 set-group-ID bit
        const S_ISVTX = 0o0001000; // 0001000 set-sticky bit

        // File specific bits
        const S_IFIFO  = 0o0010000;
        const S_IFCHR  = 0o0020000;
        const S_IFDIR  = 0o0040000;
        const S_IFBLK  = 0o0060000;
        const S_IFREG  = 0o0100000;
        const S_IFLNK  = 0o0120000;
        const S_IFSOCK = 0o0140000;
        const S_IFMT   = 0o0170000;
    }
}

/// Attempts to open the fd at the path specified by a null terminated string, with the provided `OpenFlags`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// See above, errors are converted into an Err with the corresponding error code
pub fn open(path: impl AsUnixStr, flags: OpenFlags) -> crate::Result<Fd> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(OPENAT, AT_FDCWD, ptr, flags.bits()) } as i32;
        bail_on_below_zero!(res, "`OPENAT` syscall failed");
        Ok(res)
    })
}

/// Attempts to open the fd at the path specified by a null terminated string, with the provided `OpenFlags` and `Mode`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// see above
#[inline]
pub fn open_mode(path: impl AsUnixStr, flags: OpenFlags, mode: Mode) -> crate::Result<Fd> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(OPENAT, AT_FDCWD, ptr, flags.bits(), mode.bits()) } as i32;
        bail_on_below_zero!(res, "`OPENAT` syscall failed");
        Ok(res)
    })
}

/// Attempts to open a file at the specified path from the opened directory (`Fd`) with the specified `OpenFlags`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// see above
pub fn open_at(dir: Fd, path: impl AsUnixStr, flags: OpenFlags) -> crate::Result<Fd> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(OPENAT, dir, ptr, flags.bits()) } as i32;
        bail_on_below_zero!(res, "`OPENAT` syscall failed");
        Ok(res)
    })
}

/// Attempts to open a file at the specified path from the opened directory (`Fd`) with the specified `OpenFlags` and `Mode`
/// See the [linux docs here](https://man7.org/linux/man-pages/man2/open.2.html)
/// # Errors
/// see above
pub fn open_at_mode(
    dir: Fd,
    path: impl AsUnixStr,
    flags: OpenFlags,
    mode: Mode,
) -> crate::Result<Fd> {
    path.exec_with_self_as_ptr(|ptr| {
        let res = unsafe { syscall!(OPENAT, dir, ptr, flags.bits(), mode.bits()) } as i32;
        bail_on_below_zero!(res, "`OPENAT` syscall failed");
        Ok(res)
    })
}

#[cfg(test)]
mod tests {
    use crate::unistd::open::{open, OpenFlags};

    #[test]
    fn try_open() {
        // TODO: fix
        #[cfg(feature = "alloc")]
        let path = "test-files/can_open.txt";
        #[cfg(not(feature = "alloc"))]
        let path = "test-files/can_open.txt\0";
        try_open_at_path(path).unwrap();
    }

    fn try_open_at_path(path: &str) -> crate::Result<()> {
        let _fd = open(path, OpenFlags::O_RDONLY)?;
        let _fd = open(path, OpenFlags::O_WRONLY)?;
        let _fd = open(path, OpenFlags::O_RDWR)?;
        Ok(())
    }

    // Differences between tmp-file creating on `x86_64` and `aarch64`, pretty interesting
    // seems that we can't just name a dir on `aarch64` because it produces `EISDIR`
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn try_open_temp() {
        // TODO: fix
        #[cfg(feature = "alloc")]
        let path = "test-files";
        #[cfg(not(feature = "alloc"))]
        let path = "test-files\0";
        try_open_temp_at_path(path).unwrap();
    }

    #[cfg(target_arch = "x86_64")]
    fn try_open_temp_at_path(path: &str) -> crate::Result<()> {
        use crate::unistd::open::{open_mode, Mode};
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
