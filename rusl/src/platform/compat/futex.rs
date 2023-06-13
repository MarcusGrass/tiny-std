use crate::platform::numbers::NonNegativeI32;
use linux_rust_bindings::futex::{FUTEX_CLOCK_REALTIME, FUTEX_PRIVATE_FLAG};

transparent_bitflags! {
    pub struct FutexFlags: NonNegativeI32 {
        const DEFAULT = NonNegativeI32::comptime_checked_new(0);
        const PRIVATE = NonNegativeI32::comptime_checked_new(FUTEX_PRIVATE_FLAG);
        const CLOCK_REALTIME = NonNegativeI32::comptime_checked_new(FUTEX_CLOCK_REALTIME);
    }
}
