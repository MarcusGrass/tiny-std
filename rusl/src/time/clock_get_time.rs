use sc::syscall;

use crate::platform::TimeSpec;
use crate::time::ClockId;
use crate::Result;

/// Gets the system clock time. Epoch time in seconds and nanos as seen by the system.
/// Sensitive to changes of the system clock.
/// The possible errors from this syscall are specified in the [Linux docs](https://man7.org/linux/man-pages/man3/clock_gettime.3.html)
/// It has no error cases assuming the pointer passed in is inside accessible address space
/// and the `ClockId` is a System-V style positive value
#[inline]
#[must_use]
pub fn clock_get_real_time() -> TimeSpec {
    let mut ts = core::mem::MaybeUninit::uninit();
    unsafe {
        syscall!(
            CLOCK_GETTIME,
            ClockId::CLOCK_REALTIME.bits(),
            ts.as_mut_ptr()
        );
        ts.assume_init()
    }
}

/// Gets system monotonic time. Always increasing, ie. it guarantees that
/// if called twice, the first `TimeSpec` will be smaller than the second.
/// Same as the above regarding error cases
#[inline]
#[must_use]
pub fn clock_get_monotonic_time() -> TimeSpec {
    let mut ts = core::mem::MaybeUninit::uninit();
    unsafe {
        syscall!(
            CLOCK_GETTIME,
            ClockId::CLOCK_MONOTONIC.bits(),
            ts.as_mut_ptr()
        );
        ts.assume_init()
    }
}

/// Get the timespec of the specified `ClockId`
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man3/clock_gettime.3.html)
/// # Errors
/// See above
#[inline]
pub fn clock_get_time(clock_id: ClockId) -> Result<TimeSpec> {
    let mut ts = core::mem::MaybeUninit::zeroed();
    let res = unsafe { syscall!(CLOCK_GETTIME, clock_id.bits(), ts.as_mut_ptr()) };
    bail_on_below_zero!(res, "`CLOCK_GETTIME` syscall failed");
    Ok(unsafe { ts.assume_init() })
}
