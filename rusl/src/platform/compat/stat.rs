use crate::platform::numbers::NonNegativeI32;
use crate::platform::TimeSpec;

pub type Stat = linux_rust_bindings::stat::stat;

#[repr(transparent)]
#[derive(Debug)]
pub struct Statx(pub(crate) linux_rust_bindings::stat::statx);

impl Statx {
    #[inline]
    #[must_use]
    pub const fn mask(&self) -> StatxMask {
        StatxMask(self.0.stx_mask)
    }

    #[inline]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.0.stx_size
    }

    /// Last access
    #[inline]
    #[must_use]
    pub const fn access_time(&self) -> TimeSpec {
        let ts = self.0.stx_atime;
        TimeSpec::new(ts.tv_sec, ts.tv_nsec as i64)
    }

    /// Creation
    #[inline]
    #[must_use]
    pub const fn birth_time(&self) -> TimeSpec {
        let ts = self.0.stx_btime;
        TimeSpec::new(ts.tv_sec, ts.tv_nsec as i64)
    }

    /// Content modification
    #[inline]
    #[must_use]
    pub const fn modified_time(&self) -> TimeSpec {
        let ts = self.0.stx_mtime;
        TimeSpec::new(ts.tv_sec, ts.tv_nsec as i64)
    }

    /// Metadata modification
    #[inline]
    #[must_use]
    pub const fn changed_time(&self) -> TimeSpec {
        let ts = self.0.stx_ctime;
        TimeSpec::new(ts.tv_sec, ts.tv_nsec as i64)
    }
}

transparent_bitflags! {
    pub struct StatxMask: u32 {
        const DEFAULT = 0;
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
    pub struct StatxFlags: NonNegativeI32 {
        const DEFAULT = NonNegativeI32::comptime_checked_new(0);
        const AT_SYMLINK_FOLLOW = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_SYMLINK_FOLLOW);
        const AT_NO_AUTOMOUNT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_NO_AUTOMOUNT);
        const AT_EMPTY_PATH = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_EMPTY_PATH);
        const AT_STATX_SYNC_TYPE = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_STATX_SYNC_TYPE);
        const AT_STATX_SYNC_AS_STAT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_STATX_SYNC_AS_STAT);
        const AT_STATX_FORCE_SYNC = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_STATX_FORCE_SYNC);
        const AT_STATX_DONT_SYNC = NonNegativeI32::comptime_checked_new(linux_rust_bindings::fcntl::AT_STATX_DONT_SYNC);
    }
}

/// Mode accepted by the [open syscall](https://man7.org/linux/man-pages/man2/open.2.html)
transparent_bitflags! {
    pub struct Mode: u32 {
        const DEFAULT = 0;
        const S_IRWXU = linux_rust_bindings::stat::S_IRWXU as u32; // 00700 user read write exec
        const S_IRUSR = linux_rust_bindings::stat::S_IRUSR as u32; // 00400 user Read
        const S_IWUSR = linux_rust_bindings::stat::S_IWUSR as u32; // 00200 user write
        const S_IXUSR = linux_rust_bindings::stat::S_IXUSR as u32;  // 00100 user execute
        const S_IRWXG = linux_rust_bindings::stat::S_IRWXG as u32;  // 00070 group read write exec
        const S_IRGRP = linux_rust_bindings::stat::S_IRGRP as u32;  // 00040 group read
        const S_IWGRP = linux_rust_bindings::stat::S_IWGRP as u32;  // 00020 group write
        const S_IXGRP = linux_rust_bindings::stat::S_IXGRP as u32;   // 00010 group exec
        const S_IRWXO = linux_rust_bindings::stat::S_IRWXO as u32;   // 00007 other read write exec
        const S_IROTH = linux_rust_bindings::stat::S_IROTH as u32;   // 00004 other read
        const S_IWOTH = linux_rust_bindings::stat::S_IWOTH as u32;   // 00002 other write
        const S_IXOTH = linux_rust_bindings::stat::S_IXOTH as u32;   // 00001 other execute

        // Linux specific bits
        const S_ISUID = linux_rust_bindings::stat::S_ISUID as u32; // 0004000 set-user-ID bit
        const S_ISGID = linux_rust_bindings::stat::S_ISGID as u32; // 0002000 set-group-ID bit
        const S_ISVTX = linux_rust_bindings::stat::S_ISVTX as u32; // 0001000 set-sticky bit

        // File specific bits
        const S_IFIFO  = linux_rust_bindings::stat::S_IFIFO as u32;
        const S_IFCHR  = linux_rust_bindings::stat::S_IFCHR as u32;
        const S_IFDIR  = linux_rust_bindings::stat::S_IFDIR as u32;
        const S_IFBLK  = linux_rust_bindings::stat::S_IFBLK as u32;
        const S_IFREG  = linux_rust_bindings::stat::S_IFREG as u32;
        const S_IFLNK  = linux_rust_bindings::stat::S_IFLNK as u32;
        const S_IFSOCK = linux_rust_bindings::stat::S_IFSOCK as u32;
        const S_IFMT   = linux_rust_bindings::stat::S_IFMT as u32;
    }
}

impl From<u32> for Mode {
    #[inline]
    fn from(value: u32) -> Self {
        Mode(value)
    }
}
