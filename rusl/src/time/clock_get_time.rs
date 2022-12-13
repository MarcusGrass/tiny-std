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

#[cfg(test)]
mod tests {
    use linux_rust_bindings::EINVAL;
    use crate::time::{clock_get_monotonic_time, clock_get_real_time, clock_get_time, ClockId};

    #[test]
    fn get_monotonic() {
        let t1 = clock_get_monotonic_time();
        let t2 = clock_get_monotonic_time();
        assert!(t1 < t2);
    }

    #[test]
    fn get_real() {
        let t1 = clock_get_real_time();
        let t2 = clock_get_real_time();
        assert!(t1 < t2);
    }

    #[test]
    fn get_clock_specified() {
        let t1_mono = clock_get_monotonic_time();
        let t2_mono = clock_get_time(ClockId::CLOCK_MONOTONIC).unwrap();
        assert!(t2_mono > t1_mono);
        // Same clock type, if the time between these two calls takes more than one second
        // something is way off
        assert!(t2_mono.seconds() - t2_mono.seconds() <= 1);
        let t1_real = clock_get_real_time();
        let t2_real = clock_get_time(ClockId::CLOCK_REALTIME).unwrap();
        assert!(t2_real > t1_real);
        assert!(t2_real.seconds() - t2_real.seconds() <= 1);
        expect_errno!(EINVAL, clock_get_time(ClockId::from(99999)));
    }
}
