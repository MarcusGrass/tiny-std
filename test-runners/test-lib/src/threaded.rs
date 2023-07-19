use crate::run_test;
use alloc::vec::Vec;
use core::time::Duration;

pub(crate) fn run_threaded_tests() {
    run_test!(thread_panics);
    run_test!(thread_parallel_compute);
    run_test!(thread_shared_memory_sequential_count);
    run_test!(thread_shared_memory_parallel_count);
    run_test!(thread_shared_memory_parallel_count_no_join);
    run_test!(thread_shared_memory_parallel_count_panics);
    run_test!(thread_shared_memory_parallel_count_panics_no_join);
}

fn thread_panics() {
    let t = tiny_std::thread::spawn(|| panic!("This is expected!")).unwrap();
    assert!(t.join().is_none());
}

const THREAD_LOOP_COUNT: usize = 1_000;

fn thread_parallel_compute() {
    let run_for = THREAD_LOOP_COUNT;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let handle = tiny_std::thread::spawn(move || {
            tiny_std::thread::sleep(Duration::from_millis(2)).unwrap();
            15
        })
        .unwrap();
        handles.push(handle);
    }
    let mut sum = 0;
    for handle in handles {
        sum += handle.join().unwrap();
    }
    assert_eq!(run_for * 15, sum);
}

fn thread_shared_memory_sequential_count() {
    let run_for = THREAD_LOOP_COUNT;
    let mut sum = 0;
    for _ in 0..run_for {
        let handle = tiny_std::thread::spawn(move || 1).unwrap();
        sum += handle.join().unwrap();
    }
    assert_eq!(run_for, sum);
}

fn thread_shared_memory_parallel_count() {
    let cnt = alloc::sync::Arc::new(tiny_std::sync::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
        })
        .unwrap();
        handles.push(handle);
    }
    for handle in handles {
        assert!(handle.join().is_some());
    }
    assert_eq!(run_for, *cnt.read());
}

fn thread_shared_memory_parallel_count_no_join() {
    let cnt = alloc::sync::Arc::new(tiny_std::sync::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let _handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
        })
        .unwrap();
    }
    while *cnt.read() < run_for {}
}

fn thread_shared_memory_parallel_count_panics() {
    let cnt = alloc::sync::Arc::new(tiny_std::sync::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    let mut handles = Vec::with_capacity(run_for);
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
            panic!("Die")
        })
        .unwrap();
        handles.push(handle);
    }
    for handle in handles {
        assert!(handle.join().is_none());
    }
    assert_eq!(run_for, *cnt.read());
}

fn thread_shared_memory_parallel_count_panics_no_join() {
    let cnt = alloc::sync::Arc::new(tiny_std::sync::RwLock::new(0));
    let run_for = THREAD_LOOP_COUNT;
    for _ in 0..run_for {
        let cnt_c = cnt.clone();
        let _handle = tiny_std::thread::spawn(move || {
            *cnt_c.write() += 1;
            panic!("Die")
        })
        .unwrap();
    }
    while run_for > *cnt.read() {}
    assert_eq!(run_for, *cnt.read());
}
