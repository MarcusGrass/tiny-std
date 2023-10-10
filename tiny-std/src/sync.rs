pub(crate) mod mutex;
pub(crate) mod rwlock;

use core::marker::PhantomData;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;
use rusl::error::Errno;
use rusl::futex::futex_wait;
use rusl::platform::FutexFlags;
pub use {mutex::Mutex, mutex::MutexGuard};
pub use {rwlock::RwLock, rwlock::RwLockReadGuard, rwlock::RwLockWriteGuard};

pub(crate) struct NotSend(PhantomData<*const ()>);

impl NotSend {
    #[inline]
    const fn new() -> Self {
        Self(PhantomData)
    }
}

unsafe impl Sync for NotSend {}

#[inline]
pub(crate) fn futex_wait_fast(futex: &AtomicU32, expect: u32) {
    loop {
        if futex.load(Relaxed) != expect {
            return;
        }
        match futex_wait(futex, expect, FutexFlags::PRIVATE, None) {
            Ok(()) => {
                return;
            }
            Err(e) => {
                if let Some(code) = e.code {
                    if code == Errno::EINTR {
                    } else {
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::futex_wait_fast;
    use core::sync::atomic::AtomicU32;
    use rusl::futex::futex_wake;

    #[test]
    fn wait_fast_shortcircuit() {
        let futex = AtomicU32::new(0);
        // We would expect this to hang forever if it didn't work
        futex_wait_fast(&futex, 1);
    }

    #[test]
    fn wait_fast_cant_short() {
        let futex = std::sync::Arc::new(AtomicU32::new(0));
        let f_c = futex.clone();
        let handle = std::thread::spawn(move || {
            futex_wait_fast(&f_c, 0);
        });
        // Just try wake until we wake the thread
        while futex_wake(&futex, 1).unwrap() == 0 {}
        handle.join().unwrap();
    }
}
