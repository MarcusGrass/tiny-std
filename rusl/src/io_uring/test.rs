use core::mem::MaybeUninit;

use linux_rust_bindings::errno::ETIME;

use crate::error::Errno;
use crate::io_uring::{
    io_uring_enter, io_uring_register_buf, io_uring_register_files, io_uring_register_io_slices,
    io_uring_setup, setup_io_uring,
};
use crate::network::{bind, connect, listen, socket};
use crate::platform::{
    AddressFamily, Fd, IoSliceMut, IoUring, IoUringCompletionQueueEntry, IoUringEnterFlags,
    IoUringParamFlags, IoUringParams, IoUringSQEFlags, IoUringSubmissionQueueEntry, Mode,
    OpenFlags, RenameFlags, SocketAddress, SocketType, StatxFlags, StatxMask, TimeSpec,
    AT_REMOVEDIR,
};
use crate::string::unix_str::UnixStr;
use crate::time::clock_get_monotonic_time;
use crate::unistd::{close, open, read, stat, unlink_flags};

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
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let mut bytes = [0u8; 1024];
    let buf = IoSliceMut::new(&mut bytes);
    let mut slices = [buf];
    let fd = open("test-files/can_open.txt\0", OpenFlags::O_RDONLY).unwrap();
    let user_data = 15;
    let entry = IoUringSubmissionQueueEntry::new_readv(
        fd,
        0,
        slices.as_mut_ptr() as usize,
        1,
        user_data,
        IoUringSQEFlags::empty(),
    );
    let cqe = write_await_single_entry(&mut uring, entry, user_data);
    assert_eq!(5, cqe.0.res, "Bad user data in cqe {:?}", cqe);
    assert_eq!(
        "open\n",
        core::str::from_utf8(&bytes[..cqe.0.res as usize]).unwrap()
    );
}

#[test]
fn uring_single_open() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let path = unsafe { UnixStr::from_str_unchecked("test-files/can_open.txt\0") };
    let user_data = 25;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_openat(
            None,
            path,
            OpenFlags::O_RDONLY,
            Mode::empty(),
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    let cqe = write_await_single_entry(&mut uring, entry, user_data);
    let fd = cqe.0.res as Fd;
    let mut bytes = [0u8; 1024];
    let read_bytes = read(fd, &mut bytes).unwrap();
    assert_eq!(5, read_bytes);
    assert_eq!(
        "open\n",
        core::str::from_utf8(&bytes[..read_bytes]).unwrap()
    );
}

#[test]
fn uring_single_close() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let fd = open("test-files/can_open.txt\0", OpenFlags::O_RDONLY).unwrap();
    let user_data = 35;
    let entry = IoUringSubmissionQueueEntry::new_close(fd, user_data, IoUringSQEFlags::empty());
    write_await_single_entry(&mut uring, entry, user_data);
    let Err(e) = close(fd) else {
        panic!("Uring close operation failed, expected `EBADF` on manual close after.")
    };
    assert_eq!(Errno::EBADF, e.code.unwrap())
}

#[test]
fn uring_single_statx() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let user_data = 927;
    let mut statx_uninit = MaybeUninit::uninit();
    let path = unsafe { UnixStr::from_str_unchecked("test-files/can_open.txt\0") };
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_statx(
            None,
            path,
            StatxFlags::empty(),
            StatxMask::STATX_SIZE,
            statx_uninit.as_mut_ptr(),
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    write_await_single_entry(&mut uring, entry, user_data);
    let statx = unsafe { statx_uninit.assume_init() };
    assert_eq!(5, statx.0.stx_size);
}

#[test]
fn uring_single_unlinkat() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let path =
        unsafe { UnixStr::from_str_unchecked("test-files/io_uring/can_create_remove.txt\0") };

    // Ensure file exists before test
    if let Err(e) = stat(path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
        let fd = open(path, OpenFlags::O_CREAT).unwrap();
        close(fd).unwrap();
        let stat_res = stat(path).unwrap();
        assert_eq!(0, stat_res.st_size);
    }
    let user_data = 555;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_unlink_at(
            None,
            path,
            false,
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    write_await_single_entry(&mut uring, entry, user_data);
    if let Err(e) = stat(path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
    } else {
        panic!("Expected missing file after unlink");
    }
}

#[test]
fn uring_single_rename_at() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let old_path = unsafe { UnixStr::from_str_unchecked("test-files/io_uring/move_me.txt\0") };
    let new_path = unsafe { UnixStr::from_str_unchecked("test-files/io_uring/moved.txt\0") };

    // Ensure file exists before test
    if let Err(e) = stat(old_path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
        let fd = open(old_path, OpenFlags::O_CREAT).unwrap();
        close(fd).unwrap();
        let stat_res = stat(old_path).unwrap();
        assert_eq!(0, stat_res.st_size);
    }
    let user_data = 367;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_rename_at(
            None,
            None,
            old_path,
            new_path,
            RenameFlags::empty(),
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    write_await_single_entry(&mut uring, entry, user_data);
    if let Err(e) = stat(old_path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
        let stat_new = stat(new_path).unwrap();
        assert_eq!(0, stat_new.st_size);
    } else {
        panic!("Expected missing file after rename");
    }
}

#[test]
fn uring_single_mkdir_at() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let new_dir_path =
        unsafe { UnixStr::from_str_unchecked("test-files/io_uring/test_create_dir\0") };
    // Ensure dir doesn't exist before test
    if let Err(e) = stat(new_dir_path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
    } else {
        unlink_flags(new_dir_path, AT_REMOVEDIR).unwrap();
    }
    let user_data = 1000;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_mkdirat(
            None,
            new_dir_path,
            Mode::empty(),
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    write_await_single_entry(&mut uring, entry, user_data);
    let stat = stat(new_dir_path).unwrap();
    assert_eq!(
        Mode::S_IFDIR,
        Mode::from(stat.st_mode) & Mode::S_IFMT,
        "Expected dir, got something else {stat:?}"
    );
}

#[test]
fn uring_single_socket() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let user_data = 10001;
    let entry = IoUringSubmissionQueueEntry::new_socket(
        AddressFamily::AF_UNIX,
        SocketType::SOCK_STREAM | SocketType::SOCK_CLOEXEC,
        0,
        user_data,
        IoUringSQEFlags::empty(),
    );
    write_await_single_entry(&mut uring, entry, user_data);
}

#[test]
fn uring_single_accept() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let server_socket = socket(AddressFamily::AF_UNIX, SocketType::SOCK_STREAM, 0).unwrap();
    let sock_path =
        unsafe { UnixStr::from_str_unchecked("test-files/io_uring/test-sock-accept\0") };
    let addr = SocketAddress::try_from_unix(sock_path).unwrap();
    // Ensure socket doesn't exist before test
    if let Err(e) = stat(sock_path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
    } else {
        unlink_flags(sock_path, 0).unwrap();
    }
    bind(server_socket, &addr).unwrap();
    listen(server_socket, 100).unwrap();
    let user_data = 10011;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_accept(
            server_socket,
            &addr,
            SocketType::SOCK_CLOEXEC | SocketType::SOCK_NONBLOCK,
            user_data,
            // Run as async since we know we won't be able to connect yet
            IoUringSQEFlags::IOSQE_ASYNC,
        )
    };
    // We actually have to handle this async since we're on a single thread and accept will block
    // for a connect
    let next_slot = uring.get_next_sqe_slot().unwrap();
    unsafe { next_slot.write(entry) }
    uring.flush_submission_queue();
    io_uring_enter(uring.fd, 1, 0, IoUringEnterFlags::empty()).unwrap();
    let conn_sock = socket(AddressFamily::AF_UNIX, SocketType::SOCK_STREAM, 0).unwrap();
    connect(conn_sock, &addr).unwrap();
    io_uring_enter(uring.fd, 0, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {:?}", cqe);
    assert!(cqe.0.res >= 0, "Failed res for cqe: {cqe:?}");
}

fn write_await_single_entry(
    uring: &mut IoUring,
    entry: IoUringSubmissionQueueEntry,
    user_data: u64,
) -> &IoUringCompletionQueueEntry {
    let next_slot = uring.get_next_sqe_slot().unwrap();
    unsafe {
        next_slot.write(entry);
    }
    uring.flush_submission_queue();
    io_uring_enter(uring.fd, 1, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {:?}", cqe);
    assert!(cqe.0.res >= 0, "Failed res for cqe: {cqe:?}");
    cqe
}

#[test]
fn uring_single_timeout() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    // 10 millis
    let wait_nsec = 10_000_000;
    let ts = TimeSpec::new(0, wait_nsec);
    let user_data = 27;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_timeout(
            &ts,
            true,
            None,
            user_data,
            // Run as async since we know we won't be able to connect yet
            IoUringSQEFlags::empty(),
        )
    };
    let next_slot = uring.get_next_sqe_slot().unwrap();
    unsafe {
        next_slot.write(entry);
    }
    uring.flush_submission_queue();
    let start = clock_get_monotonic_time();
    io_uring_enter(uring.fd, 1, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    let end = clock_get_monotonic_time();
    let diff = end.nanoseconds() - start.nanoseconds();
    assert!(
        diff > wait_nsec && diff - wait_nsec < 2 * wait_nsec,
        "diff {diff} isn't between {wait_nsec} and 2 * {wait_nsec}"
    );
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {:?}", cqe);
    assert_eq!(0 - ETIME, cqe.0.res, "Expected `ETIME` for cqe: {cqe:?}");
}

#[test]
fn uring_multi_linked_crud() {
    const CREATE_DIR_DATA: u64 = 1;
    const CREATE_FILE_DATA: u64 = 2;
    const STAT_FILE_DATA: u64 = 3;
    const REMOVE_FILE_DATA: u64 = 4;
    const REMOVE_DIR_DATA: u64 = 5;
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let dir_path = unsafe { UnixStr::from_str_unchecked("test-files/io_uring/multi-dir\0") };
    if stat(dir_path).is_ok() {
        unlink_flags(dir_path, AT_REMOVEDIR).unwrap();
    }
    let file_path =
        unsafe { UnixStr::from_str_unchecked("test-files/io_uring/multi-dir/new_file.txt\0") };
    let mut statx_uninit = MaybeUninit::uninit();

    unsafe {
        let mkdir = IoUringSubmissionQueueEntry::new_mkdirat(
            None,
            dir_path,
            Mode::S_IRUSR
                | Mode::S_IRGRP
                | Mode::S_IROTH
                | Mode::S_IWUSR
                | Mode::S_IWGRP
                | Mode::S_IXUSR
                | Mode::S_IXGRP
                | Mode::S_IXOTH,
            CREATE_DIR_DATA,
            IoUringSQEFlags::IOSQE_IO_LINK,
        );
        let create_file = IoUringSubmissionQueueEntry::new_openat(
            // Would use above dir but it's not created yet
            None,
            file_path,
            OpenFlags::O_CREAT | OpenFlags::O_RDWR,
            Mode::empty(),
            CREATE_FILE_DATA,
            IoUringSQEFlags::IOSQE_IO_LINK,
        );
        let stat_file = IoUringSubmissionQueueEntry::new_statx(
            None,
            file_path,
            StatxFlags::empty(),
            StatxMask::STATX_SIZE,
            statx_uninit.as_mut_ptr(),
            STAT_FILE_DATA,
            IoUringSQEFlags::IOSQE_IO_LINK,
        );
        let remove_file = IoUringSubmissionQueueEntry::new_unlink_at(
            None,
            file_path,
            false,
            REMOVE_FILE_DATA,
            IoUringSQEFlags::IOSQE_IO_LINK,
        );
        let remove_dir = IoUringSubmissionQueueEntry::new_unlink_at(
            None,
            dir_path,
            true,
            REMOVE_DIR_DATA,
            IoUringSQEFlags::IOSQE_IO_LINK,
        );
        let next = uring.get_next_sqe_slot().unwrap();
        next.write(mkdir);
        let next = uring.get_next_sqe_slot().unwrap();
        next.write(create_file);
        let next = uring.get_next_sqe_slot().unwrap();
        next.write(stat_file);
        let next = uring.get_next_sqe_slot().unwrap();
        next.write(remove_file);
        let next = uring.get_next_sqe_slot().unwrap();
        next.write(remove_dir);
    }
    uring.flush_submission_queue();

    io_uring_enter(uring.fd, 5, 5, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    for _ in 0..5 {
        let cqe = uring.get_next_cqe().unwrap();
        assert!(cqe.0.res >= 0);
        match cqe.0.user_data {
            STAT_FILE_DATA => unsafe {
                assert_eq!(0, (*statx_uninit.as_ptr()).0.stx_size);
            },
            _ => {}
        }
    }
}
