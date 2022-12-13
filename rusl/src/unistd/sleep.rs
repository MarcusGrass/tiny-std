use sc::syscall;
use crate::platform::TimeSpec;

/// Attempt to sleep for the provided `try_sleep` duration.
/// If interrupted by signal, and `rem` is provided, the os will populate it with the remaining
/// wait time.
/// See the [Linux doc for details](https://man7.org/linux/man-pages/man2/nanosleep.2.html)
/// # Errors
/// See above
pub fn nanosleep(try_sleep: &TimeSpec, rem: Option<*mut TimeSpec>) -> crate::Result<()> {
    let res = unsafe { syscall!(NANOSLEEP, try_sleep as *const TimeSpec, rem.map_or(core::ptr::null_mut(), |ts| ts))};
    bail_on_below_zero!(res, "`NANOSLEEP` syscall failed");
    Ok(())
}
