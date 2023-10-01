#[cfg(feature = "integration-test")]
use std::mem::MaybeUninit;

#[cfg(feature = "integration-test")]
use rusl::error::Errno;
#[cfg(feature = "integration-test")]
use rusl::platform::{
    Clone3Args, CloneArgs, CloneFlags, Fd, PollEvents, PollFd, SignalKind, WaitPidFlags,
};
#[cfg(feature = "integration-test")]
use rusl::process::{clone, clone3, exit, wait_pid};
#[cfg(feature = "integration-test")]
use rusl::select::ppoll;

#[test]
#[cfg(feature = "integration-test")]
fn test_clone3_vfork() {
    unsafe {
        // Doing a vfork, that's not explicitly implemented on aarch64 but is possible
        // through clone3
        let mut pidfd: MaybeUninit<Fd> = MaybeUninit::uninit();
        let mut args = Clone3Args::new(CloneFlags::CLONE_VFORK);
        args.set_pid_fd(pidfd.as_mut_ptr());
        let child = match clone3(&mut args) {
            Ok(pid) => pid as i32,
            Err(ref e) if e.code == Some(Errno::ENOSYS) => {
                return;
            }
            Err(e) => panic!("Test failure {e}"),
        };
        if child == 0 {
            exit(0);
        } else {
            let pidfd = pidfd.assume_init();
            // Pidfd is ready to read on complete
            let done_when = PollFd::new(pidfd, PollEvents::POLLOUT);
            let completed = ppoll(&mut [done_when], None, None).unwrap();
            assert_eq!(1, completed);
        }
    }
}

#[test]
#[cfg(feature = "integration-test")]
fn test_clone3_pidfd() {
    unsafe {
        let mut pidfd: MaybeUninit<Fd> = MaybeUninit::uninit();
        // Same as above but we're spawning an LVP
        let mut args = Clone3Args::new(CloneFlags::CLONE_PIDFD);
        args.set_pid_fd(pidfd.as_mut_ptr());

        // In my container, clone3 gives an `ENOSYS` seems to be fairly common container behaviour
        // to keep the sandbox under management
        let child = match clone3(&mut args) {
            Ok(pid) => pid as i32,
            Err(ref e) if e.code == Some(Errno::ENOSYS) => {
                return;
            }
            Err(e) => panic!("Test failure {e}"),
        };
        if child == 0 {
            exit(0);
        } else {
            let pidfd = pidfd.assume_init();
            // Pidfd is ready to read on complete
            let done_when = PollFd::new(pidfd, PollEvents::POLLOUT | PollEvents::POLLIN);
            let completed = ppoll(&mut [done_when], None, None).unwrap();
            assert_eq!(1, completed);
        }
    }
}

#[test]
#[cfg(feature = "integration-test")]
fn test_regular_clone_vfork() {
    unsafe {
        let flags = CloneFlags::CLONE_VFORK;
        let mut args = CloneArgs::new(flags);
        args
            // Needs to be explicitly set on aarch64 or we'll EINVAL
            .set_exit(SignalKind::SIGCHLD);
        let child = clone(&args).unwrap();
        if child == 0 {
            exit(0);
        } else {
            let res = wait_pid(child, WaitPidFlags::empty()).unwrap();
            assert_eq!(0, res.status);
            assert_eq!(child, res.pid);
        }
    }
}
