pub use clock_get_time::{clock_get_monotonic_time, clock_get_real_time, clock_get_time};

mod clock_get_time;

transparent_bitflags!(
    pub struct ClockId: i32 {
        const CLOCK_REALTIME = 0;
        const CLOCK_MONOTONIC = 1;
        const CLOCK_PROCESS_CPUTIME_ID = 2;
        const CLOCK_THREAD_CPUTIME_ID = 3;
        const CLOCK_MONOTONIC_RAW = 4;
        const CLOCK_REALTIME_COARSE = 5;
        const CLOCK_MONOTONIC_COARSE = 6;
        const CLOCK_BOOTTIME = 7;
        const CLOCK_REALTIME_ALARM = 8;
        const CLOCK_BOOTTIME_ALARM = 9;
        const CLOCK_TAI = 11;
    }
);
