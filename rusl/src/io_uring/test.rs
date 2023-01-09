use crate::io_uring::{
    io_uring_enter, io_uring_register_buf, io_uring_register_files, io_uring_register_io_slices,
    io_uring_setup, setup_io_uring,
};
use crate::platform::{
    Fd, IoSliceMut, IoUring, IoUringEnterFlags, IoUringParamFlags, IoUringParams, IoUringSQEFlags,
    IoUringSubmissionQueueEntry, Mode, OpenFlags,
};
use crate::string::unix_str::UnixStr;
use crate::unistd::{open, read};

#[test]
fn uring_setup() {
    let _ = setup_io_poll_uring();
}

fn setup_io_poll_uring() -> Option<Fd> {
    let mut params = IoUringParams::new(IoUringParamFlags::IORING_SETUP_IOPOLL);
    let uring_fd = match io_uring_setup(1, &mut params) {
        Ok(uring_fd) => {
            assert_ne!(0, uring_fd);
            uring_fd
        }
        #[allow(unused_variables)]
        Err(e) => {
            #[cfg(target_arch = "aarch64")]
            if e.code.unwrap() != crate::error::Errno::ENOSYS {
                panic!("{}", e);
            }
            return None;
        }
    };
    Some(uring_fd)
}

#[test]
fn uring_register_files() {
    let Some(uring_fd) = setup_io_poll_uring() else {
        return;
    };
    let open_fd = open("test-files/can_open.txt\0", OpenFlags::O_RDWR).unwrap();
    io_uring_register_files(uring_fd, &[open_fd]).unwrap();
}

#[test]
fn uring_register_io_slices() {
    let Some(uring_fd) = setup_io_poll_uring() else {
        return;
    };
    let mut buf1 = [0; 1024];
    let mut buf2 = [0; 1024];
    let io_slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
    io_uring_register_io_slices(uring_fd, &io_slices).unwrap();
}

#[test]
fn uring_register_buffer() {
    let Some(uring_fd) = setup_io_poll_uring() else {
        return;
    };
    let buf1 = [0; 1024];
    io_uring_register_buf(uring_fd, &buf1).unwrap();
}

fn setup_ignore_enosys(entries: u32, flags: IoUringParamFlags) -> Option<IoUring> {
    let uring = setup_io_uring(entries, flags);
    match uring {
        Ok(u) => Some(u),
        Err(e) => {
            #[cfg(target_arch = "aarch64")]
            if e.code.unwrap() == crate::error::Errno::ENOSYS {
                return None;
            }
            panic!("{e}")
        }
    }
}

#[test]
fn uring_setup_instance() {
    let Some(uring) = setup_ignore_enosys(8, IoUringParamFlags::IORING_SETUP_SQPOLL) else {
        return;
    };
    let res = io_uring_enter(uring.fd, 0, 0, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    assert_eq!(0, res);
}

#[test]
fn uring_single_read() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::IORING_SETUP_SQPOLL) else {
        return;
    };
    let next_slot = uring.get_next_sqe_slot().unwrap();
    let mut bytes = [0u8; 1024];
    let buf = IoSliceMut::new(&mut bytes);
    let mut slices = [buf];
    let fd = open("test-files/can_open.txt\0", OpenFlags::O_RDONLY).unwrap();
    let user_data = 15;
    unsafe {
        next_slot.write(IoUringSubmissionQueueEntry::new_readv(
            fd,
            0,
            slices.as_mut_ptr() as usize,
            1,
            user_data,
            IoUringSQEFlags::empty(),
        ))
    }
    uring.flush_submission_queue();
    io_uring_enter(
        uring.fd,
        1,
        1,
        IoUringEnterFlags::IORING_ENTER_GETEVENTS | IoUringEnterFlags::IORING_ENTER_SQ_WAKEUP,
    )
    .unwrap();
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {:?}", cqe);
    assert_eq!(5, cqe.0.res, "Bad user data in cqe {:?}", cqe);
    assert_eq!(
        "open\n",
        core::str::from_utf8(&bytes[..cqe.0.res as usize]).unwrap()
    );
}

#[test]
fn uring_single_open() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::IORING_SETUP_SQPOLL) else {
        return;
    };
    let next_slot = uring.get_next_sqe_slot().unwrap();
    let path = unsafe { UnixStr::from_str_unchecked("test-files/can_open.txt\0") };
    let user_data = 25;
    unsafe {
        next_slot.write(IoUringSubmissionQueueEntry::new_openat(
            None,
            path,
            OpenFlags::O_RDONLY,
            Mode::empty(),
            user_data,
            IoUringSQEFlags::empty(),
        ))
    }
    uring.flush_submission_queue();
    io_uring_enter(
        uring.fd,
        1,
        1,
        IoUringEnterFlags::IORING_ENTER_GETEVENTS | IoUringEnterFlags::IORING_ENTER_SQ_WAKEUP,
    )
    .unwrap();
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {:?}", cqe);
    assert!(cqe.0.res > 0, "open failed for cqe: {cqe:?}");
    let fd = cqe.0.res as Fd;
    let mut bytes = [0u8; 1024];
    let read_bytes = read(fd, &mut bytes).unwrap();
    assert_eq!(5, read_bytes);
    assert_eq!(
        "open\n",
        core::str::from_utf8(&bytes[..read_bytes]).unwrap()
    );
}
