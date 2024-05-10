use crate::error::Errno;
use crate::platform::{IoSlice, IoSliceMut, Mode, OpenFlags};
use crate::string::unix_str::UnixStr;
use crate::unistd::read::readv;
use crate::unistd::write::writev;
use crate::unistd::{
    close, fcntl_get_file_status, fcntl_set_file_status, open, open_mode, read, unlink, write,
};

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

#[test]
fn can_read_write() {
    const CONTENT: &[u8; 21] = b"Test write into file\n";
    let path = unix_lit!("test-files/unistd/read_write.txt");
    let _ = unlink(path);
    let fd = open_mode(
        path,
        OpenFlags::O_WRONLY | OpenFlags::O_CREAT,
        Mode::MODE_755,
    )
    .unwrap();
    write(fd, CONTENT).unwrap();
    close(fd).unwrap();
    let fd = open(path, OpenFlags::O_RDONLY).unwrap();
    let mut buf = [0u8; CONTENT.len()];
    read(fd, &mut buf).unwrap();
    assert_eq!(&buf, CONTENT);
}

#[test]
fn can_read_writev() {
    const PART_A: &[u8] = b"First line\n";
    const PART_B: &[u8] = b"Second line\nThird line\n";
    const PART_C: &[u8] = b"Fourth line\n";
    let iova = IoSlice::new(PART_A);
    let iovb = IoSlice::new(PART_B);
    let iovc = IoSlice::new(PART_C);
    let path = unix_lit!("test-files/unistd/read_writev.txt");
    let _ = unlink(path);
    let fd = open_mode(
        path,
        OpenFlags::O_WRONLY | OpenFlags::O_CREAT,
        Mode::MODE_755,
    )
    .unwrap();
    writev(fd, &[iova, iovb, iovc]).unwrap();
    close(fd).unwrap();
    let mut recva = [0u8; PART_A.len()];
    let mut recvb = [0u8; PART_B.len()];
    let mut recvc = [0u8; PART_C.len()];
    let iovra = IoSliceMut::new(&mut recva);
    let iovrb = IoSliceMut::new(&mut recvb);
    let iovrc = IoSliceMut::new(&mut recvc);

    let fd = open(path, OpenFlags::O_RDONLY).unwrap();
    readv(fd, &mut [iovra, iovrb, iovrc]).unwrap();

    assert_eq!(recva, PART_A);
    assert_eq!(recvb, PART_B);
    assert_eq!(recvc, PART_C);
}
