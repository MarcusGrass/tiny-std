use linux_rust_bindings::futex::{FUTEX_CLOCK_REALTIME, FUTEX_PRIVATE_FLAG};

transparent_bitflags! {
    pub struct FutexFlags: i32 {
        const PRIVATE = FUTEX_PRIVATE_FLAG;
        const CLOCK_REALTIME = FUTEX_CLOCK_REALTIME;
    }
}
