use crate::process::{exit, fork, wait_pid};

#[test]
fn fork_test() {
    // Just gonna fork and die in the subproc
    unsafe {
        let res = fork().unwrap();
        if res == 0 {
            panic!("I'm dead but it doesn't matter because I was forked!");
        }
    }
}

#[test]
fn fork_then_wait() {
    unsafe {
        let child = fork().unwrap();
        if child != 0 {
            // Parent path, waits
            let res = wait_pid(child, 0).unwrap();
            assert_eq!(0, res.status);
            assert_eq!(child, res.pid);
        } else {
            // Child path, just exits
            exit(0);
        }
    }
}
