use crate::platform::TimeSpec;
use sc::syscall;

/// Attempt to sleep for the provided `try_sleep` duration.
/// If interrupted by signal, and `rem` is provided, the os will populate it with the remaining
/// wait time.
/// See the [Linux doc for details](https://man7.org/linux/man-pages/man2/nanosleep.2.html)
/// # Errors
/// See above
#[inline]
pub fn nanosleep(try_sleep: &TimeSpec, rem: Option<*mut TimeSpec>) -> crate::Result<()> {
    let res = unsafe {
        syscall!(
            NANOSLEEP,
            core::ptr::from_ref::<TimeSpec>(try_sleep),
            rem.map_or(core::ptr::null_mut(), |ts| ts)
        )
    };
    bail_on_below_zero!(res, "`NANOSLEEP` syscall failed");
    Ok(())
}

/// Same as above, except in the case of an EINTR, the result is written into the provided `TimeSpec`.
/// See the [Linux doc for details](https://man7.org/linux/man-pages/man2/nanosleep.2.html)
/// # Errors
/// See above
#[inline]
pub fn nanosleep_same_ptr(try_sleep: &mut TimeSpec) -> crate::Result<()> {
    let res = unsafe {
        syscall!(
            NANOSLEEP,
            core::ptr::from_mut::<TimeSpec>(try_sleep),
            core::ptr::from_mut::<TimeSpec>(try_sleep)
        )
    };
    bail_on_below_zero!(res, "`NANOSLEEP` syscall failed");
    Ok(())
}
