use core::time::Duration;

use linux_rust_bindings::time::__kernel_timespec;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct ClockId(pub(crate) i32);

impl ClockId {
    pub const CLOCK_REALTIME: Self = Self(linux_rust_bindings::time::CLOCK_REALTIME);
    pub const CLOCK_MONOTONIC: Self = Self(linux_rust_bindings::time::CLOCK_MONOTONIC);
    pub const CLOCK_PROCESS_CPUTIME_ID: Self =
        Self(linux_rust_bindings::time::CLOCK_PROCESS_CPUTIME_ID);
    pub const CLOCK_THREAD_CPUTIME_ID: Self =
        Self(linux_rust_bindings::time::CLOCK_THREAD_CPUTIME_ID);
    pub const CLOCK_MONOTONIC_RAW: Self = Self(linux_rust_bindings::time::CLOCK_MONOTONIC_RAW);
    pub const CLOCK_REALTIME_COARSE: Self = Self(linux_rust_bindings::time::CLOCK_REALTIME_COARSE);
    pub const CLOCK_MONOTONIC_COARSE: Self =
        Self(linux_rust_bindings::time::CLOCK_MONOTONIC_COARSE);
    pub const CLOCK_BOOTTIME: Self = Self(linux_rust_bindings::time::CLOCK_BOOTTIME);
    pub const CLOCK_REALTIME_ALARM: Self = Self(linux_rust_bindings::time::CLOCK_REALTIME_ALARM);
    pub const CLOCK_BOOTTIME_ALARM: Self = Self(linux_rust_bindings::time::CLOCK_BOOTTIME_ALARM);
    pub const CLOCK_TAI: Self = Self(linux_rust_bindings::time::CLOCK_TAI);

    #[inline]
    #[must_use]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn into_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for ClockId {
    #[inline]
    fn from(value: i32) -> Self {
        ClockId(value)
    }
}

/// `__kernel_timespec` is the type going over the syscall layer
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TimeSpec(__kernel_timespec);

impl TimeSpec {
    #[inline]
    #[must_use]
    pub const fn new_zeroed() -> Self {
        Self(__kernel_timespec {
            tv_sec: 0,
            tv_nsec: 0,
        })
    }

    #[inline]
    #[must_use]
    pub fn new(seconds: i64, nanoseconds: i64) -> Self {
        Self(__kernel_timespec {
            tv_sec: seconds,
            tv_nsec: nanoseconds,
        })
    }

    #[inline]
    #[must_use]
    pub fn seconds(&self) -> i64 {
        self.0.tv_sec
    }

    #[inline]
    #[must_use]
    pub fn nanoseconds(&self) -> i64 {
        self.0.tv_nsec
    }
}

impl Default for TimeSpec {
    #[inline]
    fn default() -> Self {
        Self::new_zeroed()
    }
}

impl TryFrom<Duration> for TimeSpec {
    type Error = crate::Error;

    #[inline]
    fn try_from(d: Duration) -> Result<Self, Self::Error> {
        Ok(TimeSpec(__kernel_timespec {
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
        }))
    }
}
