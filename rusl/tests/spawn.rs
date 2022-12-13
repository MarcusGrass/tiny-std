use std::mem::MaybeUninit;
use rusl::platform::Fd;
use rusl::process::{clone3, CloneArgs, CloneFlags, exit};
use rusl::select::{PollEvents, PollFd, ppoll};

#[test]
fn test_clone3_vfork() {
    unsafe {
        // Doing a vfork, that's not explicitly implemented on aarch64 but is possible
        // through clone3
        let mut pidfd: MaybeUninit<Fd> = MaybeUninit::uninit();
        let mut args = CloneArgs::new(CloneFlags::CLONE_VFORK);
        args.set_pid_fd(pidfd.as_mut_ptr());
        let child = clone3(&mut args).unwrap() as i32;
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
        let mut args = CloneArgs::new(CloneFlags::CLONE_VM);
        args.set_pid_fd(pidfd.as_mut_ptr())
            .set_stack(&mut child_stack);

        let child = clone3(&mut args).unwrap() as i32;
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