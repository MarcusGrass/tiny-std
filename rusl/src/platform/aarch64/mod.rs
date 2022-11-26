use crate::platform::{BlkCntT, BlkSizeT, DevT, GidT, InoT, NlinkT, OffT, TimeSpec, UidT};
use crate::unistd::Mode;

/// Field ordering differs between `aarch64` and `x86_64`
/// [stat, fstat, lstat, fstatat - get file status](https://man7.org/linux/man-pages/man2/statx.2.html)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Stat {
    /* ID of device containing file */
    pub st_dev: DevT,
    /* Inode number */
    pub st_ino: InoT,
    /* File type and mode */
    pub st_mode: Mode,
    /* Number of hard links */
    pub st_nlink: NlinkT,
    /* User ID of owner */
    pub st_uid: UidT,
    /* Group ID of owner */
    pub st_gid: GidT,
    __pad: u32,
    /* Device ID (if special file) */
    pub st_rdev: DevT,
    /* Total size, in bytes */
    pub st_size: OffT,
    /* Block size for filesystem I/O */
    pub st_blksize: BlkSizeT,
    /* Number of 512B blocks allocated */
    pub st_blocks: BlkCntT,

    /* Since Linux 2.6, the kernel supports nanosecond
    precision for the following timestamp fields.
    For the details before Linux 2.6, see NOTES. */
    pub st_atime: TimeSpec,
    pub st_mtime: TimeSpec,
    pub st_ctime: TimeSpec,
    __reserve: [i64; 3],
}
