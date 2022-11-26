use core::time::Duration;

pub mod signal;

/// Shared typedefs for 64 bit systems (GNU source)
pub type UidT = u32;
pub type GidT = u32;
pub type PidT = i32;
pub type Fd = i32;
pub type OffT = i64;
pub type BlkSizeT = i64;
pub type BlkCntT = i64;
pub type DevT = u64;
pub type InoT = u64;
pub type Ino64T = u64;
pub type NlinkT = u64;

/// Shared constants for 64 bit systems
pub const AT_FDCWD: i32 = -100;
pub const AT_REMOVEDIR: i32 = 0x200;
pub const AT_SYMLINK_NOFOLLOW: i32 = 0x400;
pub const AT_NO_AUTOMOUNT: i32 = 0x800;
pub const AT_EMPTY_PATH: i32 = 0x1000;

/// File types
pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;

/// Shared between sock and open
pub const O_NONBLOCK: i32 = 2048;
pub const O_CLOEXEC: i32 = 0x80000;

#[repr(C)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TimeSpec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

impl TryFrom<Duration> for TimeSpec {
    type Error = crate::Error;

    #[inline]
    fn try_from(d: Duration) -> Result<Self, Self::Error> {
        Ok(TimeSpec {
            tv_sec: d.as_secs().try_into().map_err(|_| {
                crate::Error::no_code("Failed to fit duration u64 secs into tv_sec i64")
            })?,
            tv_nsec: d
                .subsec_nanos()
                .try_into()
                // This one doesn't make a lot of sense
                .map_err(|_| {
                    crate::Error::no_code("Failed to fit duration u32 secs into tv_sec i32")
                })?,
        })
    }
}

// Directory entity
#[repr(C)]
pub struct DirEnt {
    pub d_ino: InoT,
    pub d_off: OffT,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
