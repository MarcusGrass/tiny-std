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

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
