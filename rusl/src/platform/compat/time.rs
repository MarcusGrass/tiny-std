use core::cmp::Ordering;
use core::time::Duration;

use linux_rust_bindings::time::__kernel_timespec;

transparent_bitflags!(
    pub struct ClockId: i32 {
        const CLOCK_REALTIME = linux_rust_bindings::time::CLOCK_REALTIME;
        const CLOCK_MONOTONIC = linux_rust_bindings::time::CLOCK_MONOTONIC;
        const CLOCK_PROCESS_CPUTIME_ID = linux_rust_bindings::time::CLOCK_PROCESS_CPUTIME_ID;
        const CLOCK_THREAD_CPUTIME_ID = linux_rust_bindings::time::CLOCK_THREAD_CPUTIME_ID;
        const CLOCK_MONOTONIC_RAW = linux_rust_bindings::time::CLOCK_MONOTONIC_RAW;
        const CLOCK_REALTIME_COARSE = linux_rust_bindings::time::CLOCK_REALTIME_COARSE;
        const CLOCK_MONOTONIC_COARSE = linux_rust_bindings::time::CLOCK_MONOTONIC_COARSE;
        const CLOCK_BOOTTIME = linux_rust_bindings::time::CLOCK_BOOTTIME;
        const CLOCK_REALTIME_ALARM = linux_rust_bindings::time::CLOCK_REALTIME_ALARM;
        const CLOCK_BOOTTIME_ALARM = linux_rust_bindings::time::CLOCK_BOOTTIME_ALARM;
        const CLOCK_TAI = linux_rust_bindings::time::CLOCK_TAI;
    }
);

/// `__kernel_timespec` is the type going over the syscall layer
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
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

impl PartialEq for TimeSpec {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.tv_sec.eq(&other.0.tv_sec) && self.0.tv_nsec.eq(&other.0.tv_nsec)
    }
}

impl Eq for TimeSpec {}

impl PartialOrd for TimeSpec {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimeSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.tv_sec.cmp(&other.0.tv_sec) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => match self.0.tv_nsec.cmp(&other.0.tv_nsec) {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Greater,
            },
            Ordering::Greater => Ordering::Greater,
        }
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
