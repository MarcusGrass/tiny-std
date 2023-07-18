/// We'll need symbols to set up a `panic_handler`, and alloc to run threads.
/// We also need to set tls, which is done in `start`
#[cfg(all(feature = "start", feature = "symbols", feature = "alloc"))]
pub(crate) mod spawn;

use crate::error::Result;
use core::time::Duration;
use rusl::error::Errno;
#[cfg(all(feature = "start", feature = "symbols", feature = "alloc"))]
pub use spawn::*;

/// Sleep the current thread for the provided `Duration`
/// # Errors
/// Errors on a malformed duration, or userspace data copying errors
pub fn sleep(duration: Duration) -> Result<()> {
    let mut ts = duration.try_into()?;
    loop {
        match rusl::time::nanosleep_same_ptr(&mut ts) {
            Ok(_) => return Ok(()),
            Err(ref e) if e.code == Some(Errno::EINTR) => {
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use crate::thread::sleep;
    use crate::time::MonotonicInstant;

    #[test]
    fn try_sleep() {
        let sleep_dur = Duration::from_nanos(5_000_000);
        let now = MonotonicInstant::now();
        sleep(sleep_dur).unwrap();
        let elapsed = now.elapsed();
        let drift = elapsed - sleep_dur;
        assert!(drift.as_millis() <= 1);
    }
}
