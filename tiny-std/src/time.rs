use core::time::Duration;

use rusl::platform::TimeSpec;

#[cfg(test)]
mod test;

pub const UNIX_TIME: SystemTime = SystemTime(TimeSpec::new_zeroed());

/// Some monotonic, ever increasing, instant in time. Cannot be manipulated and is only good
/// for comparing elapsed time.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MonotonicInstant(TimeSpec);

const NANOS_A_SECOND: i64 = 1_000_000_000;

impl MonotonicInstant {
    /// Create a new instant
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        Self(get_monotonic_time())
    }

    /// Get the time that has passed since this instant
    /// Will always yield a valid `Duration` and never panic
    #[must_use]
    pub fn elapsed(self) -> Duration {
        sub_ts_dur(Self::now().0, self.0)
    }

    /// Converts this `MonotonicInstant` into a regular `Instant`
    #[must_use]
    #[inline]
    pub fn as_instant(self) -> Instant {
        Instant(self.0)
    }
}

/// Some instant in time, ever increasing but able to be manipulated.
/// The manipulations carries a risk of over/underflow,
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Instant(TimeSpec);

impl Instant {
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        Self(get_monotonic_time())
    }

    /// Get the time that has passed since this instant.
    /// If this `Instant` is by some manipulation after `now`, returns `None`
    #[inline]
    #[must_use]
    pub fn elapsed(self) -> Option<Duration> {
        Self::now().duration_since(self)
    }

    /// Get the duration since some other `Instant`.
    /// If this `Instant` is before `other` in monotonic time, returns `None`
    #[must_use]
    pub fn duration_since(self, other: Self) -> Option<Duration> {
        sub_ts_checked_dur(self.0, other.0)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Option<Self>;

    fn add(self, rhs: Duration) -> Self::Output {
        checked_add_dur(self.0, rhs).map(Self)
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Option<Self>;

    fn sub(self, rhs: Duration) -> Self::Output {
        checked_sub_dur(self.0, rhs).map(Self)
    }
}

impl core::ops::Sub for Instant {
    type Output = Option<Duration>;

    fn sub(self, rhs: Self) -> Self::Output {
        sub_ts_checked_dur(self.0, rhs.0)
    }
}

/// Some instant in time since the unix epoch as seen by the system
/// The passage of time may not be linear
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SystemTime(TimeSpec);

impl SystemTime {
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        Self(get_real_time())
    }

    #[inline]
    #[must_use]
    pub fn elapsed(self) -> Option<Duration> {
        Self::now().duration_since(self)
    }

    #[must_use]
    pub fn duration_since(self, other: Self) -> Option<Duration> {
        sub_ts_checked_dur(self.0, other.0)
    }

    #[must_use]
    pub fn duration_since_unix_time(self) -> Duration {
        sub_ts_dur(self.0, UNIX_TIME.0)
    }
}

impl core::ops::Add<Duration> for SystemTime {
    type Output = Option<Self>;

    fn add(self, rhs: Duration) -> Self::Output {
        checked_add_dur(self.0, rhs).map(Self)
    }
}

impl core::ops::Sub<Duration> for SystemTime {
    type Output = Option<Self>;

    fn sub(self, rhs: Duration) -> Self::Output {
        checked_sub_dur(self.0, rhs).map(Self)
    }
}

impl core::ops::Sub for SystemTime {
    type Output = Option<Duration>;

    fn sub(self, rhs: Self) -> Self::Output {
        sub_ts_checked_dur(self.0, rhs.0)
    }
}

#[inline]
fn checked_add_dur(timespec: TimeSpec, duration: Duration) -> Option<TimeSpec> {
    // tv_nsec are < `NANOS_A_SECOND`, this cannot overflow
    let mut total_nanos = timespec
        .nanoseconds()
        .checked_add(duration.subsec_nanos().into())?;
    let mut seconds = duration.as_secs();
    if total_nanos >= NANOS_A_SECOND {
        total_nanos -= NANOS_A_SECOND;
        seconds = seconds.checked_add(1)?;
    };
    Some(TimeSpec::new(
        timespec.seconds().checked_add(seconds.try_into().ok()?)?,
        total_nanos,
    ))
}

#[inline]
fn checked_sub_dur(timespec: TimeSpec, duration: Duration) -> Option<TimeSpec> {
    let mut total_nanos = timespec
        .nanoseconds()
        .checked_sub(duration.subsec_nanos().into())?;
    let mut seconds = duration.as_secs();
    if total_nanos < 0 {
        // tv_nsec is always < `NANOS_A_SECOND`, so this won't get wonky
        total_nanos += NANOS_A_SECOND;
        seconds = seconds.checked_sub(1)?;
    }
    let tv_sec = timespec.seconds().checked_sub(seconds.try_into().ok()?)?;

    Some(TimeSpec::new(tv_sec.gt(&0).then_some(tv_sec)?, total_nanos))
}

#[inline]
fn sub_ts_dur(lhs: TimeSpec, rhs: TimeSpec) -> Duration {
    let mut total_nanos = lhs.nanoseconds() - rhs.nanoseconds();
    let sub_sec = if total_nanos < 0 {
        // tv_nsec are < `NANOS_A_SECOND`, so this won't get wonky
        total_nanos += NANOS_A_SECOND;
        1
    } else {
        0
    };
    let secs = (lhs.seconds() - rhs.seconds() - sub_sec) as u64;
    let nanos = total_nanos as u32;
    Duration::new(secs, nanos)
}

#[inline]
fn sub_ts_checked_dur(lhs: TimeSpec, rhs: TimeSpec) -> Option<Duration> {
    let mut total_nanos = lhs.nanoseconds().checked_sub(rhs.nanoseconds())?;
    let sub_sec = if total_nanos < 0 {
        // tv_nsec are < `NANOS_A_SECOND`, so this won't get wonky
        total_nanos += NANOS_A_SECOND;
        1
    } else {
        0
    };
    let secs = u64::try_from(
        lhs.seconds()
            .checked_sub(rhs.seconds())?
            .checked_sub(sub_sec)?,
    )
    .ok()?;
    let nanos = u32::try_from(total_nanos).ok()?;
    Some(Duration::new(secs, nanos))
}

#[inline]
#[cfg(feature = "vdso")]
fn get_monotonic_time() -> TimeSpec {
    if let Some(vdso_get_time) = unsafe { crate::start::VDSO_CLOCK_GET_TIME } {
        let mut ts = core::mem::MaybeUninit::<TimeSpec>::zeroed();
        vdso_get_time(
            rusl::platform::ClockId::CLOCK_MONOTONIC.bits(),
            ts.as_mut_ptr(),
        );
        unsafe {
            return ts.assume_init();
        }
    }
    rusl::time::clock_get_monotonic_time()
}

#[inline]
#[cfg(feature = "vdso")]
fn get_real_time() -> TimeSpec {
    if let Some(vdso_get_time) = unsafe { crate::start::VDSO_CLOCK_GET_TIME } {
        let mut ts = core::mem::MaybeUninit::<TimeSpec>::zeroed();
        vdso_get_time(
            rusl::platform::ClockId::CLOCK_REALTIME.bits(),
            ts.as_mut_ptr(),
        );
        unsafe {
            return ts.assume_init();
        }
    }
    rusl::time::clock_get_real_time()
}

#[inline]
#[cfg(not(feature = "vdso"))]
fn get_monotonic_time() -> TimeSpec {
    rusl::time::clock_get_monotonic_time()
}

#[inline]
#[cfg(not(feature = "vdso"))]
fn get_real_time() -> TimeSpec {
    rusl::time::clock_get_real_time()
}
