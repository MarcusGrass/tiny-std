use std::mem::MaybeUninit;
use linux_rust_bindings::{EINVAL, ENOSYS};
use rusl::platform::Fd;
use rusl::process::{clone, clone3, Clone3Args, CloneArgs, CloneFlags, exit, wait_pid};
use rusl::select::{PollEvents, PollFd, ppoll};

#[test]
fn test_clone3_vfork() {
    unsafe {
        // Doing a vfork, that's not explicitly implemented on aarch64 but is possible
        // through clone3
        let mut pidfd: MaybeUninit<Fd> = MaybeUninit::uninit();
        let mut args = Clone3Args::new(CloneFlags::CLONE_VFORK);
        args.set_pid_fd(pidfd.as_mut_ptr());
        let child = match clone3(&mut args) {
            Ok(pid) => pid as i32,
            Err(ref e) if e.code == Some(ENOSYS) => {
                return;
            }
            Err(e) => panic!("Test failure {e}"),
        };
        if child != 0 {
            let pidfd = pidfd.assume_init();
            // Pidfd is ready to read on complete
            let done_when = PollFd::new(pidfd, PollEvents::POLLOUT);
            let completed = ppoll(&mut [done_when], None, None).unwrap();
            assert_eq!(1, completed);
        } else {
            exit(0);
        }
    }
}

#[test]
fn test_clone3_new_thread() {
    unsafe {
        let mut pidfd: MaybeUninit<Fd> = MaybeUninit::uninit();
        // Same as above but we're spawning an LVP
        let mut child_stack = [0u8; 1048];
        let mut args = Clone3Args::new(CloneFlags::CLONE_VM & CloneFlags::CLONE_PIDFD);
        args.set_pid_fd(pidfd.as_mut_ptr())
            .set_stack(&mut child_stack);

        // In my container, clone3 gives an `ENOSYS` seems to be fairly common container behaviour
        // to keep the sandbox under management
        let child = match clone3(&mut args) {
            Ok(pid) => pid as i32,
            Err(ref e) if e.code == Some(ENOSYS) => {
                return;
            }
            Err(e) => panic!("Test failure {e}"),
        };
        if child != 0 {
            let pidfd = pidfd.assume_init();
            // Pidfd is ready to read on complete
            let done_when = PollFd::new(pidfd, PollEvents::POLLOUT);
            let completed = ppoll(&mut [done_when], None, None).unwrap();
            assert_eq!(1, completed);
        } else {
            exit(0);
        }
    }
}

#[test]
fn test_regular_clone_new_thread() {
    unsafe {
        // Same as above but we're spawning an LVP
        let mut child_stack = [0u8; 4096];
        let mut args = CloneArgs::new(CloneFlags::CLONE_VM);
        args.set_stack(&mut child_stack);

        // In my container, clone3 gives an `ENOSYS` seems to be fairly common container behaviour
        // to keep the sandbox under management
        let child = clone(&args).unwrap();
        if child != 0 {
            let res = wait_pid(child, 0).unwrap();
            assert_eq!(child, res.pid);
            assert_eq!(0, res.status);
        }
    }
}