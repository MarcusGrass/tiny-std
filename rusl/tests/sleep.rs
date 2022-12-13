use core::time::Duration;

#[test]
fn test_sleep() {
    const NANO_COEF: usize = 1_000_000_000;
    let start = rusl::time::clock_get_monotonic_time();
    let sleep_nanos = 5_000_000;
    let sleep_for = Duration::from_nanos(sleep_nanos);
    let ts = sleep_for.try_into().unwrap();
    rusl::unistd::nanosleep(&ts, None).unwrap();
    let end = rusl::time::clock_get_monotonic_time();
    let start_nanos = start.seconds() as usize * NANO_COEF + start.nanoseconds() as usize;
    let end_nanos = end.seconds() as usize * NANO_COEF + end.nanoseconds() as usize;
    // A bit of slack time here, 1 milli acceptable drift for the test
    assert!(end_nanos - start_nanos < sleep_nanos as usize + 1_000_000);
}