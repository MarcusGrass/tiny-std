use crate::platform::WaitPidFlags;
use crate::process::{exit, fork, wait_pid};

#[test]
fn fork_test() {
    // Just gonna fork and die in the subproc
    unsafe {
        let res = fork().unwrap();
        if res == 0 {
            exit(0);
        }
    }
}

#[test]
fn fork_then_wait() {
    unsafe {
        let child = fork().unwrap();
        if child == 0 {
            // Child path, just exits
            exit(0);
        } else {
            // Parent path, waits
            let res = wait_pid(child, WaitPidFlags::empty()).unwrap();
            assert_eq!(0, res.status);
            assert_eq!(child, res.pid);
        }
    }
}
