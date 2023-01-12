pub type Stat = linux_rust_bindings::stat::stat;

#[repr(transparent)]
#[derive(Debug)]
pub struct Statx(pub(crate) linux_rust_bindings::stat::statx);

transparent_bitflags! {
    pub struct StatxMask: u32 {
        const STATX_TYPE = linux_rust_bindings::stat::STATX_TYPE as u32;
        const STATX_MODE = linux_rust_bindings::stat::STATX_MODE as u32;
        const STATX_NLINK = linux_rust_bindings::stat::STATX_NLINK as u32;
        const STATX_UID = linux_rust_bindings::stat::STATX_UID as u32;
        const STATX_GID = linux_rust_bindings::stat::STATX_GID as u32;
        const STATX_ATIME = linux_rust_bindings::stat::STATX_ATIME as u32;
        const STATX_MTIME = linux_rust_bindings::stat::STATX_MTIME as u32;
        const STATX_CTIME = linux_rust_bindings::stat::STATX_CTIME as u32;
        const STATX_INO = linux_rust_bindings::stat::STATX_INO as u32;
        const STATX_SIZE = linux_rust_bindings::stat::STATX_SIZE as u32;
        const STATX_BLOCKS = linux_rust_bindings::stat::STATX_BLOCKS as u32;
        const STATX_BASIC_STATS = linux_rust_bindings::stat::STATX_BASIC_STATS as u32;
        const STATX_BTIME = linux_rust_bindings::stat::STATX_BTIME as u32;
        const STATX_MNT_ID = linux_rust_bindings::stat::STATX_MNT_ID as u32;
        const STATX_DIOALIGN = linux_rust_bindings::stat::STATX_DIOALIGN as u32;
    }
}

transparent_bitflags! {
    pub struct StatxFlags: i32 {
        const AT_SYMLINK_FOLLOW = linux_rust_bindings::fcntl::AT_SYMLINK_FOLLOW;
        const AT_NO_AUTOMOUNT = linux_rust_bindings::fcntl::AT_NO_AUTOMOUNT;
        const AT_EMPTY_PATH = linux_rust_bindings::fcntl::AT_EMPTY_PATH;
        const AT_STATX_SYNC_TYPE = linux_rust_bindings::fcntl::AT_STATX_SYNC_TYPE;
        const AT_STATX_SYNC_AS_STAT = linux_rust_bindings::fcntl::AT_STATX_SYNC_AS_STAT;
        const AT_STATX_FORCE_SYNC = linux_rust_bindings::fcntl::AT_STATX_FORCE_SYNC;
        const AT_STATX_DONT_SYNC = linux_rust_bindings::fcntl::AT_STATX_DONT_SYNC;
    }
}
