use crate::platform::numbers::NonNegativeI32;

#[derive(Debug, Copy, Clone)]
pub enum FcntlFileStatusCmd {
    Get,
    Set,
}

impl FcntlFileStatusCmd {
    pub(crate) const fn into_cmd(self) -> i32 {
        match self {
            Self::Get => linux_rust_bindings::fcntl::F_GETFL,
            Self::Set => linux_rust_bindings::fcntl::F_SETFL,
        }
    }
}

pub const AT_FDCWD: i32 = linux_rust_bindings::fcntl::AT_FDCWD;
pub const AT_REMOVEDIR: NonNegativeI32 =
    NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_REMOVEDIR);

/// Openflags accepted by the [open syscall](https://man7.org/linux/man-pages/man2/open.2.html)
transparent_bitflags! {
    pub struct OpenFlags: NonNegativeI32 {
        const DEFAULT = NonNegativeI32::comptime_checked_new(0);
        const O_RDONLY = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_RDONLY);
        const O_WRONLY = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_WRONLY);
        const O_RDWR = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_RDWR);
        const O_APPEND = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_APPEND);
        const O_ASYNC = NonNegativeI32::comptime_checked_new(0x2000);
        const O_CLOEXEC = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_CLOEXEC);
        const O_CREAT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_CREAT);
        const O_DIRECT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_DIRECT);
        const O_DIRECTORY = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_DIRECTORY);
        const O_DSYNC = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_DSYNC);
        const O_EXCL = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_EXCL);
        const O_LARGEFILE = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_LARGEFILE);
        const O_NOATIME = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_NOATIME);
        const O_NOCTTY = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_NOCTTY);
        const O_NOFOLLOW = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_NOFOLLOW);
        const O_NONBLOCK = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_NONBLOCK);
        const O_NDELAY = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_NDELAY);
        const O_PATH = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_PATH);
        const O_SYNC = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_SYNC);
        const O_TMPFILE = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_TMPFILE);
        const O_TRUNC = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::O_TRUNC);
    }
}

transparent_bitflags! {
    pub struct DirFlags: NonNegativeI32 {
        const DEFAULT = NonNegativeI32::comptime_checked_new(0);
        const AT_EMPTY_PATH = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_EMPTY_PATH);
        const AT_NO_AUTOMOUNT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_NO_AUTOMOUNT);
        const AT_SYMLINK_NOFOLLOW = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_SYMLINK_NOFOLLOW);
    }
}
