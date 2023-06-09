use crate::platform::{FutexFlags, TimeSpec};
use crate::Error;
use core::sync::atomic::AtomicU32;
use linux_rust_bindings::futex::{FUTEX_WAIT, FUTEX_WAKE};
use sc::syscall;

/// Runs the `FUTEX` syscall with arguments relating to the `FUTEX_WAIT` operation.
/// See [The linux documentation for details](https://man7.org/linux/man-pages/man2/futex.2.html)
/// # Errors
/// See above documentation
#[inline]
pub fn futex_wait(
    uaddr: &AtomicU32,
    val: u32,
    flags: FutexFlags,
    timeout: Option<TimeSpec>,
) -> Result<(), Error> {
    let res = unsafe {
        syscall!(
            FUTEX,
            uaddr as *const AtomicU32,
            FUTEX_WAIT & flags.bits(),
            val,
            timeout
                .as_ref()
                .map_or_else(core::ptr::null, |ts| ts as *const TimeSpec),
            0,
            0
        )
    };
    bail_on_below_zero!(res, "`FUTEX` (wait) syscall failed");
    Ok(())
}

/// Runs the `FUTEX` syscall with arguments relating to the `FUTEX_WAKE` operation.
/// See [The linux documentation for details](https://man7.org/linux/man-pages/man2/futex.2.html)
/// # Errors
/// See above documentation
#[inline]
pub fn futex_wake(uaddr: &AtomicU32, num_waiters: i32) -> Result<usize, Error> {
    let res = unsafe {
        syscall!(
            FUTEX,
            uaddr as *const AtomicU32,
            FUTEX_WAKE,
            num_waiters,
            0,
            0,
            0
        )
    };
    bail_on_below_zero!(res, "`FUTEX` (wake) syscall failed");
    Ok(res)
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "alloc")]
    fn test_futex_wait_wake() {
        use crate::futex::{futex_wait, futex_wake};
        use crate::platform::FutexFlags;
        use core::sync::atomic::AtomicU32;
        use core::time::Duration;

        let rf = alloc::sync::Arc::new(AtomicU32::new(15));
        let rf_c = rf.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            futex_wake(rf.as_ref(), i32::MAX).unwrap();
        });
        futex_wait(rf_c.as_ref(), 15, FutexFlags::empty(), None).unwrap();
    }
}
