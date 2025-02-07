use core::mem::MaybeUninit;

use linux_rust_bindings::errno::ETIME;

use crate::error::Errno;
use crate::io_uring::{
    io_uring_enter, io_uring_register_buffers, io_uring_register_files,
    io_uring_register_io_slices, io_uring_setup, setup_io_uring,
};
use crate::platform::{
    AddressFamily, Fd, IoSlice, IoSliceMut, IoUring, IoUringCompletionQueueEntry,
    IoUringEnterFlags, IoUringParamFlags, IoUringParams, IoUringSQEFlags,
    IoUringSubmissionQueueEntry, Mode, OpenFlags, PollAddMultiFlags, PollEvents, RenameFlags,
    SocketFlags, SocketOptions, SocketType, StatxFlags, StatxMask, TimeSpec, STDERR, STDIN, STDOUT,
};
use crate::string::unix_str::UnixStr;
use crate::time::clock_get_monotonic_time;
use crate::unistd::{close, open, open_mode, read, stat, unlink_flags, UnlinkFlags};

#[test]
fn uring_setup() {
    let _ = setup_io_poll_uring();
}

fn setup_io_poll_uring() -> Option<Fd> {
    let mut params = IoUringParams::new(IoUringParamFlags::IORING_SETUP_IOPOLL, 0, 0);
    let uring_fd = match io_uring_setup(1, &mut params) {
        Ok(uring_fd) => {
            assert_ne!(0, uring_fd.0);
            uring_fd
        }
        #[allow(unused_variables)]
        Err(e) => {
            #[cfg(target_arch = "aarch64")]
            assert!(e.code.unwrap() == crate::error::Errno::ENOSYS);
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
    let open_fd = open(
        UnixStr::try_from_str("test-files/can_open.txt\0").unwrap(),
        OpenFlags::O_RDWR,
    )
    .unwrap();
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
    let mut buf1 = [0; 1024];
    let ioslice = IoSliceMut::new(&mut buf1);
    unsafe { io_uring_register_buffers(uring_fd, &[ioslice]).unwrap() };
}

#[cfg_attr(target_arch = "x86_64", expect(clippy::unnecessary_wraps))]
fn setup_ignore_enosys(entries: u32, flags: IoUringParamFlags) -> Option<IoUring> {
    let uring = setup_io_uring(entries, flags, 0, 0);
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
#[expect(clippy::cast_sign_loss)]
fn uring_single_read() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let mut bytes = [0u8; 1024];
    let buf = IoSliceMut::new(&mut bytes);
    let mut slices = [buf];
    let fd = open(
        UnixStr::try_from_str("test-files/can_open.txt\0").unwrap(),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    let user_data = 15;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_readv(
            fd,
            slices.as_mut_ptr() as usize,
            1,
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    let cqe = write_await_single_entry(&mut uring, entry, user_data);
    assert_eq!(5, cqe.0.res, "Bad user data in cqe {cqe:?}");
    assert_eq!(
        "open\n",
        core::str::from_utf8(&bytes[..cqe.0.res as usize]).unwrap()
    );
}

#[test]
#[expect(clippy::cast_sign_loss)]
fn uring_single_write() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let path =
        unsafe { UnixStr::from_str_unchecked("test-files/io_uring/tmp_uring_swrite_test\0") };
    let bytes = b"Uring written!\n";
    let buf = IoSlice::new(bytes);
    let mut slices = [buf];

    let fd = open_mode(
        path,
        OpenFlags::O_RDWR | OpenFlags::O_TRUNC | OpenFlags::O_CREAT,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .unwrap();
    let user_data = 15559;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_writev(
            fd,
            slices.as_mut_ptr() as usize,
            1,
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    let cqe = write_await_single_entry(&mut uring, entry, user_data);
    assert_eq!(
        "Uring written!\n",
        core::str::from_utf8(&bytes[..cqe.0.res as usize]).unwrap()
    );
    let mut buf = [0u8; 15];
    read(fd, &mut buf).unwrap();
    assert_eq!(bytes, &buf);
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
    let fd = Fd::try_new(cqe.0.res).unwrap();
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
    let fd = open(
        UnixStr::try_from_str("test-files/can_open.txt\0").unwrap(),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    let user_data = 35;
    let entry = IoUringSubmissionQueueEntry::new_close(fd, user_data, IoUringSQEFlags::empty());
    write_await_single_entry(&mut uring, entry, user_data);
    let Err(e) = close(fd) else {
        panic!("Uring close operation failed, expected `EBADF` on manual close after.")
    };
    assert_eq!(Errno::EBADF, e.code.unwrap());
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
        unlink_flags(new_dir_path, UnlinkFlags::at_removedir()).unwrap();
    }
    let user_data = 1000;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_mkdirat(
            None,
            new_dir_path,
            Mode(0o755),
            user_data,
            IoUringSQEFlags::empty(),
        )
    };
    write_await_single_entry(&mut uring, entry, user_data);
    let stat = stat(new_dir_path).unwrap();
    assert_eq!(
        Mode::S_IFDIR,
        Mode(stat.st_mode) & Mode::S_IFMT,
        "Expected dir, got something else {stat:?}"
    );
}

#[test]
fn uring_single_socket() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    if std::env::var("CI").is_ok() {
        // Github actions doesn't allow us to open sockets through uring it seems
        return;
    }
    let user_data = 10001;
    let entry = IoUringSubmissionQueueEntry::new_socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
        user_data,
        IoUringSQEFlags::empty(),
    );
    write_await_single_entry(&mut uring, entry, user_data);
}

#[test]
#[cfg(feature = "alloc")]
#[expect(clippy::cast_sign_loss, clippy::too_many_lines)]
fn uring_unix_accept_send_recv() {
    use crate::network::{bind_unix, connect_unix, listen, socket};
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let server_socket = socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::empty()),
        0,
    )
    .unwrap();
    let sock_path =
        unsafe { UnixStr::from_str_unchecked("test-files/io_uring/test-sock-accept\0") };
    let addr = crate::platform::SocketAddressUnix::try_from_unix(sock_path).unwrap();
    // Ensure socket doesn't exist before test
    if let Err(e) = stat(sock_path) {
        assert_eq!(Errno::ENOENT, e.code.unwrap());
    } else {
        crate::unistd::unlink(sock_path).unwrap();
    }
    bind_unix(server_socket, &addr).unwrap();
    listen(
        server_socket,
        crate::platform::NonNegativeI32::comptime_checked_new(100),
    )
    .unwrap();
    let user_data = 10011;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_accept_unix(
            server_socket,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SocketFlags::SOCK_CLOEXEC | SocketFlags::SOCK_NONBLOCK,
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
    let conn_sock = socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::empty()),
        0,
    )
    .unwrap();
    connect_unix(conn_sock, &addr).unwrap();
    io_uring_enter(uring.fd, 0, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {cqe:?}");
    assert!(cqe.0.res >= 0, "Failed res for cqe: {cqe:?}");
    let client_to_server_socket = Fd::try_new(cqe.0.res).unwrap();
    let next_slot = uring.get_next_sqe_slot().unwrap();
    let msg_content = b"Ping!";
    let msg_fd = open(
        unix_lit!("test-files/io_uring/uring_recv_send_pass_fd.txt"),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    let mut file_buf_orig = [0u8; 64];
    read(msg_fd, &mut file_buf_orig).unwrap();
    let msg_fds = [msg_fd];
    unsafe {
        let io = [IoSlice::new(msg_content)];
        let msg = crate::platform::MsgHdrBorrow::create_send(
            None,
            &io,
            Some(crate::platform::ControlMessageSend::ScmRights(&msg_fds)),
        );
        *next_slot = IoUringSubmissionQueueEntry::new_sendmsg(
            conn_sock,
            &msg,
            0,
            99,
            IoUringSQEFlags::empty(),
        );
        uring.flush_submission_queue();
        io_uring_enter(uring.fd, 1, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
        let cqe = uring.get_next_cqe().unwrap();
        assert_eq!(msg_content.len(), cqe.0.res as usize);
        assert_eq!(99, cqe.0.user_data);
        let next_slot = uring.get_next_sqe_slot().unwrap();
        let mut buf = [0u8; 32];
        let mut ctrl_buf = [0u8; 126];
        let mut iov = [IoSliceMut::new(&mut buf)];
        let mut recv_hdr =
            crate::platform::MsgHdrBorrow::create_recv(&mut iov, Some(&mut ctrl_buf));
        *next_slot = IoUringSubmissionQueueEntry::new_recvmsg(
            client_to_server_socket,
            core::ptr::addr_of_mut!(recv_hdr).cast(),
            0,
            999,
            IoUringSQEFlags::empty(),
        );
        uring.flush_submission_queue();
        io_uring_enter(uring.fd, 1, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
        let cqe = uring.get_next_cqe().unwrap();
        assert!(cqe.0.res > 0, "Got error result {}", cqe.0.res);
        assert_eq!(msg_content.len(), cqe.0.res as usize);
        assert_eq!(b"Ping!", &buf[..5]);
        let mut ctrl = recv_hdr.control_messages();
        let fd = ctrl.next().unwrap();
        assert!(ctrl.next().is_none());
        match fd {
            crate::platform::ControlMessageSend::ScmRights(fds) => {
                assert_eq!(1, fds.len());
                assert!(msg_fd < fds[0]);
                // Since it's the same fd a reseek is necessary here
                let off = crate::unistd::lseek(fds[0], 0, crate::unistd::Whence::SET).unwrap();
                assert_eq!(0, off);
                let mut file_buf_passed = [0u8; 64];
                read(fds[0], &mut file_buf_passed).unwrap();
                assert_eq!(file_buf_orig, file_buf_passed);
                let expect = b"Text content!\n";
                assert_eq!(expect, &file_buf_passed[..14]);
            }
        }
    }
}

#[test]
#[cfg(feature = "alloc")]
fn uring_tcp_accept() {
    use crate::network::{bind_inet, listen, socket};
    const FIFTEEN: crate::platform::NonNegativeI32 =
        crate::platform::NonNegativeI32::comptime_checked_new(15);
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    // Dynamically assign port
    let mut addr = crate::platform::SocketAddressInet::new([0, 0, 0, 0], 0);
    let srv_sock = socket(
        AddressFamily::AF_INET,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        6,
    )
    .unwrap();
    bind_inet(srv_sock, &addr).unwrap();
    // Get port
    let sockname = crate::network::get_inet_sock_name(srv_sock).unwrap();
    addr.0.sin_port = sockname.0.sin_port;
    listen(srv_sock, FIFTEEN).unwrap();
    let user_data = 10012;
    let entry = unsafe {
        IoUringSubmissionQueueEntry::new_accept_inet(
            srv_sock,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SocketFlags::SOCK_CLOEXEC | SocketFlags::SOCK_NONBLOCK,
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
    let conn_sock = socket(
        AddressFamily::AF_INET,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        6,
    )
    .unwrap();
    crate::network::connect_inet(conn_sock, &addr).unwrap();
    io_uring_enter(uring.fd, 0, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {cqe:?}");
    assert!(cqe.0.res >= 0, "Failed res for cqe: {cqe:?}");
    let server_to_client_socket = Fd::try_new(cqe.0.res).unwrap();
    let msg_content = b"Ping!";
    crate::unistd::write(server_to_client_socket, msg_content).unwrap();
    let mut ret_buf = [0u8; 5];
    crate::unistd::read(conn_sock, &mut ret_buf).unwrap();
    assert_eq!(&ret_buf, msg_content);
    // Todo: Better cleanup on test failures
    let _ = crate::unistd::close(server_to_client_socket);
    let _ = crate::unistd::close(conn_sock);
    let _ = crate::unistd::close(srv_sock);
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
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {cqe:?}");
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
    let start_ts = i128::from(start.seconds())
        .checked_mul(1_000_000_000)
        .unwrap()
        .checked_add(i128::from(start.nanoseconds()))
        .unwrap();
    let end_ts = i128::from(end.seconds())
        .checked_mul(1_000_000_000)
        .unwrap()
        .checked_add(i128::from(end.nanoseconds()))
        .unwrap();
    let diff = end_ts - start_ts;
    assert!(
        diff >= i128::from(wait_nsec),
        "Diff failed, start = {}, end = {}, diff = {}, wait = {}",
        start.nanoseconds(),
        end.nanoseconds(),
        diff,
        wait_nsec
    );
    let cqe = uring.get_next_cqe().unwrap();
    assert_eq!(user_data, cqe.0.user_data, "Bad user data in cqe {cqe:?}");
    assert_eq!(0 - ETIME, cqe.0.res, "Expected `ETIME` for cqe: {cqe:?}");
}

#[test]
#[expect(clippy::cast_sign_loss)]
fn uring_read_registered_buffers_and_fds() {
    let mut buf1 = [0u8; 64];
    let buf1_addr = core::ptr::addr_of_mut!(buf1);
    let mut buf2 = [0u8; 64];
    let buf2_addr = core::ptr::addr_of_mut!(buf2);
    let mut buf3 = [0u8; 64];
    let buf3_addr = core::ptr::addr_of_mut!(buf3);
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    unsafe {
        io_uring_register_buffers(
            uring.fd,
            &[
                IoSliceMut::new(&mut buf1),
                IoSliceMut::new(&mut buf2),
                IoSliceMut::new(&mut buf3),
            ],
        )
        .unwrap();
    }
    let fd1 = open(
        UnixStr::try_from_str("test-files/io_uring/uring_register_read1\0").unwrap(),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    let fd2 = open(
        UnixStr::try_from_str("test-files/io_uring/uring_register_read2\0").unwrap(),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    let fd3 = open(
        UnixStr::try_from_str("test-files/io_uring/uring_register_read3\0").unwrap(),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    io_uring_register_files(uring.fd, &[fd1, fd2, fd3]).unwrap();
    unsafe {
        let r1 = IoUringSubmissionQueueEntry::new_readv_fixed(
            STDIN,
            0,
            buf1_addr as u64,
            64,
            1,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        );
        let r2 = IoUringSubmissionQueueEntry::new_readv_fixed(
            STDOUT,
            1,
            buf2_addr as u64,
            64,
            2,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        );
        let r3 = IoUringSubmissionQueueEntry::new_readv_fixed(
            STDERR,
            2,
            buf3_addr as u64,
            64,
            3,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        );
        uring.get_next_sqe_slot().unwrap().write(r1);
        uring.get_next_sqe_slot().unwrap().write(r2);
        uring.get_next_sqe_slot().unwrap().write(r3);
        uring.flush_submission_queue();
        io_uring_enter(uring.fd, 3, 3, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
        for _ in 0..3 {
            let cqe = uring.get_next_cqe().unwrap();
            assert!(cqe.0.res >= 0, "Cqe with error {cqe:?}");
            match cqe.0.user_data {
                1 => assert_eq!(
                    b"Read into first\n",
                    &buf1[..cqe.0.res as usize],
                    "bad match on cqe {cqe:?}"
                ),
                2 => assert_eq!(
                    b"Read into second\n",
                    &buf2[..cqe.0.res as usize],
                    "bad match on cqe {cqe:?}"
                ),
                3 => assert_eq!(
                    b"Read into third\n",
                    &buf3[..cqe.0.res as usize],
                    "bad match on cqe {cqe:?}"
                ),
                _ => panic!("Bad user data on cqe {cqe:?}"),
            }
        }
    }
}

#[test]
#[expect(clippy::too_many_lines)]
fn uring_write_registered_buffers_and_fds() {
    let content1 = b"Uring fixed write 1!\n";
    let mut buf1 = [0u8; 21];
    buf1.copy_from_slice(content1);
    let buf1_addr = core::ptr::addr_of_mut!(buf1);
    let content2 = b"Uring fixed write 2!\n";
    let mut buf2 = [0u8; 21];
    buf2.copy_from_slice(content2);
    let buf2_addr = core::ptr::addr_of_mut!(buf2);
    let content3 = b"Uring fixed write 3!\n";
    let mut buf3 = [0u8; 21];
    buf3.copy_from_slice(content3);
    let buf3_addr = core::ptr::addr_of_mut!(buf3);
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    unsafe {
        io_uring_register_buffers(
            uring.fd,
            &[
                IoSliceMut::new(&mut buf1),
                IoSliceMut::new(&mut buf2),
                IoSliceMut::new(&mut buf3),
            ],
        )
        .unwrap();
    }
    let path1 = UnixStr::try_from_str("test-files/io_uring/tmp_uring_register_write1\0").unwrap();
    let path2 = UnixStr::try_from_str("test-files/io_uring/tmp_uring_register_write2\0").unwrap();
    let path3 = UnixStr::try_from_str("test-files/io_uring/tmp_uring_register_write3\0").unwrap();
    let fd1 = open_mode(
        path1,
        OpenFlags::O_RDWR | OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .unwrap();
    let fd2 = open_mode(
        path2,
        OpenFlags::O_RDWR | OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .unwrap();
    let fd3 = open_mode(
        path3,
        OpenFlags::O_RDWR | OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .unwrap();
    io_uring_register_files(uring.fd, &[fd1, fd2, fd3]).unwrap();
    unsafe {
        let r1 = IoUringSubmissionQueueEntry::new_writev_fixed(
            STDIN,
            0,
            buf1_addr as u64,
            21,
            1,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        );
        let r2 = IoUringSubmissionQueueEntry::new_writev_fixed(
            STDOUT,
            1,
            buf2_addr as u64,
            21,
            2,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        );
        let r3 = IoUringSubmissionQueueEntry::new_writev_fixed(
            STDERR,
            2,
            buf3_addr as u64,
            21,
            3,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        );
        uring.get_next_sqe_slot().unwrap().write(r1);
        uring.get_next_sqe_slot().unwrap().write(r2);
        uring.get_next_sqe_slot().unwrap().write(r3);
        uring.flush_submission_queue();
        io_uring_enter(uring.fd, 3, 3, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
        for _ in 0..3 {
            let cqe = uring.get_next_cqe().unwrap();
            assert!(cqe.0.res >= 0, "Cqe with error {cqe:?}");
            match cqe.0.user_data {
                1 => {
                    let fd = open(path1, OpenFlags::O_RDONLY).unwrap();
                    let mut buf = [0u8; 21];
                    read(fd, &mut buf).unwrap();
                    assert_eq!(
                        b"Uring fixed write 1!\n", &mut buf,
                        "bad match on cqe {cqe:?}"
                    );
                }
                2 => {
                    let fd = open(path2, OpenFlags::O_RDONLY).unwrap();
                    let mut buf = [0u8; 21];
                    read(fd, &mut buf).unwrap();
                    assert_eq!(
                        b"Uring fixed write 2!\n", &mut buf,
                        "bad match on cqe {cqe:?}"
                    );
                }
                3 => {
                    let fd = open(path3, OpenFlags::O_RDONLY).unwrap();
                    let mut buf = [0u8; 21];
                    read(fd, &mut buf).unwrap();
                    assert_eq!(b"Uring fixed write 3!\n", &buf3, "bad match on cqe {cqe:?}");
                }
                _ => panic!("Bad user data on cqe {cqe:?}"),
            }
        }
    }
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
        unlink_flags(dir_path, UnlinkFlags::at_removedir()).unwrap();
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
        if cqe.0.user_data == STAT_FILE_DATA {
            unsafe {
                assert_eq!(0, (*statx_uninit.as_ptr()).0.stx_size);
            }
        }
    }
}

#[test]
fn poll_add() {
    let Some(mut uring) = setup_ignore_enosys(8, IoUringParamFlags::empty()) else {
        return;
    };
    let poll_file_path = unix_lit!("test-files/io_uring/tmp_uring_poll_add_1.txt");
    let poll_fd = open_mode(
        poll_file_path,
        OpenFlags::O_CREAT | OpenFlags::O_RDWR,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .unwrap();
    unsafe {
        let sqe = uring.get_next_sqe_slot().unwrap();
        *sqe = IoUringSubmissionQueueEntry::new_poll_add(
            poll_fd,
            PollEvents::POLLIN | PollEvents::POLLOUT,
            PollAddMultiFlags::empty(),
            0,
            IoUringSQEFlags::empty(),
        );
    }
    uring.flush_submission_queue();
    assert_eq!(
        1,
        io_uring_enter(uring.fd, 1, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap()
    );
    let next = uring.get_next_cqe().unwrap();
    assert_eq!(
        next.0.res,
        i32::from((PollEvents::POLLIN | PollEvents::POLLOUT).0)
    );
}
