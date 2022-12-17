pub const AT_FDCWD: i32 = linux_rust_bindings::fcntl::AT_FDCWD;
pub const AT_REMOVEDIR: i32 = linux_rust_bindings::fcntl::AT_REMOVEDIR;

/// Openflags accepted by the [open syscall](https://man7.org/linux/man-pages/man2/open.2.html)
transparent_bitflags! {
    pub struct OpenFlags: i32 {
        const O_RDONLY = linux_rust_bindings::fcntl::O_RDONLY;
        const O_WRONLY = linux_rust_bindings::fcntl::O_WRONLY;
        const O_RDWR = linux_rust_bindings::fcntl::O_RDWR;
        const O_APPEND = linux_rust_bindings::fcntl::O_APPEND;
        const O_ASYNC = 0x2000;
        const O_CLOEXEC = linux_rust_bindings::fcntl::O_CLOEXEC;
        const O_CREAT = linux_rust_bindings::fcntl::O_CREAT;
        const O_DIRECT = linux_rust_bindings::fcntl::O_DIRECT;
        const O_DIRECTORY = linux_rust_bindings::fcntl::O_DIRECTORY;
        const O_DSYNC = linux_rust_bindings::fcntl::O_DSYNC;
        const O_EXCL = linux_rust_bindings::fcntl::O_EXCL;
        const O_LARGEFILE = linux_rust_bindings::fcntl::O_LARGEFILE;
        const O_NOATIME = linux_rust_bindings::fcntl::O_NOATIME;
        const O_NOCTTY = linux_rust_bindings::fcntl::O_NOCTTY;
        const O_NOFOLLOW = linux_rust_bindings::fcntl::O_NOFOLLOW;
        const O_NONBLOCK = linux_rust_bindings::fcntl::O_NONBLOCK;
        const O_NDELAY = linux_rust_bindings::fcntl::O_NDELAY;
        const O_PATH = linux_rust_bindings::fcntl::O_PATH;
        const O_SYNC = linux_rust_bindings::fcntl::O_SYNC;
        const O_TMPFILE = linux_rust_bindings::fcntl::O_TMPFILE;
        const O_TRUNC = linux_rust_bindings::fcntl::O_TRUNC;
    }
}

transparent_bitflags! {
    pub struct DirFlags: i32 {
        const AT_EMPTY_PATH = linux_rust_bindings::fcntl::AT_EMPTY_PATH;
        const AT_NO_AUTOMOUNT = linux_rust_bindings::fcntl::AT_NO_AUTOMOUNT;
        const AT_SYMLINK_NOFOLLOW = linux_rust_bindings::fcntl::AT_SYMLINK_NOFOLLOW;
    }
}
