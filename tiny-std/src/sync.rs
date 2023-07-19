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
            Ok(_) => {
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
