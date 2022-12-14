use core::ops::Add;
use core::time::Duration;

use crate::time::{Instant, MonotonicInstant, SystemTime};

#[test]
fn monotonic_ever_increasing() {
    let mut last = MonotonicInstant::now();
    for _ in 0..100 {
        let cur = MonotonicInstant::now();
        assert!(cur > last);
        last = cur;
    }
}

#[test]
fn instant_ever_increasing() {
    let mut last = Instant::now();
    for _ in 0..100 {
        let cur = Instant::now();
        assert!(cur > last);
        last = cur;
    }
}

#[test]
fn instant_add_sub_cmp() {
    let first = Instant::now();
    let diff = Duration::from_secs(10);
    let later = first.add(diff).unwrap();
    assert_eq!(diff, (later - first).unwrap());
    assert_eq!(diff, later.duration_since(first).unwrap());
    assert!((first - later).is_none());
    assert!(first.duration_since(later).is_none())
}

#[test]
fn system_time_add_sub_cmp() {
    let first = SystemTime::now();
    let diff = Duration::from_secs(10);
    let later = first.add(diff).unwrap();
    assert_eq!(diff, (later - first).unwrap());
    assert_eq!(diff, later.duration_since(first).unwrap());
    assert!((first - later).is_none());
    assert!(first.duration_since(later).is_none())
}
