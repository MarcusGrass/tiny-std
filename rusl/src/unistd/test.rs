use linux_rust_bindings::EBADF;
use crate::unistd::{close, open, OpenFlags, read, write};

#[test]
fn no_write_on_read_only() {
    let fd = open("test-files/can_open.txt\0", OpenFlags::O_RDONLY).unwrap();
    let mut buf = [0; 256];
    let read_bytes = read(fd, &mut buf).unwrap();
    let expect = b"open";
    assert!(read_bytes >= expect.len());
    assert_eq!(expect, &buf[..expect.len()]);
    // No write on read only
    expect_errno!(EBADF, write(fd, &[]));
    close(fd).unwrap();
}

#[test]
fn no_read_on_wr_only() {
    let path = "test-files/can_open.txt\0";
    let fd = open(path, OpenFlags::O_WRONLY).unwrap();
    let mut buf = [0; 256];
    expect_errno!(EBADF, read(fd, &mut buf));
    assert_eq!(0, write(fd, &[]).unwrap());
    close(fd).unwrap();
    // Write on closed should fail
    expect_errno!(EBADF, write(fd, &[]));
}

#[test]
fn close_closes() {
    let path = "test-files/can_open.txt\0";
    let fd = open(path, OpenFlags::O_RDONLY).unwrap();
    let mut buf = [0; 128];
    let read_res = read(fd, &mut buf).unwrap();
    assert!(read_res > 0);
    close(fd).unwrap();
    // Read on closed should fail
    expect_errno!(EBADF, read(fd, &mut buf));
    // Close on closed should also fail
    expect_errno!(EBADF, close(fd));
}