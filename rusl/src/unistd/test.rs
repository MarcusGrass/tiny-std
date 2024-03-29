use crate::error::Errno;
use crate::platform::OpenFlags;
use crate::string::unix_str::UnixStr;
use crate::unistd::{close, fcntl_get_file_status, fcntl_set_file_status, open, read, write};

#[test]
fn no_write_on_read_only() {
    let fd = open(
        UnixStr::try_from_str("test-files/can_open.txt\0").unwrap(),
        OpenFlags::O_RDONLY,
    )
    .unwrap();
    let mut buf = [0; 256];
    let read_bytes = read(fd, &mut buf).unwrap();
    let expect = b"open";
    assert!(read_bytes >= expect.len());
    assert_eq!(expect, &buf[..expect.len()]);
    // No write on read only
    expect_errno!(Errno::EBADF, write(fd, &[]));
    close(fd).unwrap();
}

#[test]
fn no_read_on_wr_only() {
    let path = UnixStr::try_from_str("test-files/can_open.txt\0").unwrap();
    let fd = open(path, OpenFlags::O_WRONLY).unwrap();
    let mut buf = [0; 256];
    expect_errno!(Errno::EBADF, read(fd, &mut buf));
    assert_eq!(0, write(fd, &[]).unwrap());
    close(fd).unwrap();
    // Write on closed should fail
    expect_errno!(Errno::EBADF, write(fd, &[]));
}

#[test]
fn close_closes() {
    let path = UnixStr::try_from_str("test-files/can_open.txt\0").unwrap();
    let fd = open(path, OpenFlags::O_RDONLY).unwrap();
    let mut buf = [0; 128];
    let read_res = read(fd, &mut buf).unwrap();
    assert!(read_res > 0);
    close(fd).unwrap();
    // Read on closed should fail
    expect_errno!(Errno::EBADF, read(fd, &mut buf));
    // Close on closed should also fail
    expect_errno!(Errno::EBADF, close(fd));
}

#[test]
fn set_file_non_blocking() {
    let path = UnixStr::try_from_str("test-files/can_open.txt\0").unwrap();
    let fd = open(path, OpenFlags::O_RDONLY).unwrap();
    let flags = fcntl_get_file_status(fd).unwrap();
    assert_eq!(OpenFlags::empty(), flags & OpenFlags::O_NONBLOCK);
    fcntl_set_file_status(fd, OpenFlags::O_NONBLOCK).unwrap();
    let new_flags = fcntl_get_file_status(fd).unwrap();
    assert_eq!(OpenFlags::O_NONBLOCK, new_flags & OpenFlags::O_NONBLOCK);
}
