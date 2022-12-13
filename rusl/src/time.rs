pub use clock_get_time::{clock_get_monotonic_time, clock_get_real_time, clock_get_time};

mod clock_get_time;

transparent_bitflags!(
    pub struct ClockId: i32 {
        const CLOCK_REALTIME = linux_rust_bindings::CLOCK_REALTIME;
        const CLOCK_MONOTONIC = linux_rust_bindings::CLOCK_MONOTONIC;
        const CLOCK_PROCESS_CPUTIME_ID = linux_rust_bindings::CLOCK_PROCESS_CPUTIME_ID;
        const CLOCK_THREAD_CPUTIME_ID = linux_rust_bindings::CLOCK_THREAD_CPUTIME_ID;
        const CLOCK_MONOTONIC_RAW = linux_rust_bindings::CLOCK_MONOTONIC_RAW;
        const CLOCK_REALTIME_COARSE = linux_rust_bindings::CLOCK_REALTIME_COARSE;
        const CLOCK_MONOTONIC_COARSE = linux_rust_bindings::CLOCK_MONOTONIC_COARSE;
        const CLOCK_BOOTTIME = linux_rust_bindings::CLOCK_BOOTTIME;
        const CLOCK_REALTIME_ALARM = linux_rust_bindings::CLOCK_REALTIME_ALARM;
        const CLOCK_BOOTTIME_ALARM = linux_rust_bindings::CLOCK_BOOTTIME_ALARM;
        const CLOCK_TAI = linux_rust_bindings::CLOCK_TAI;
    }
);
