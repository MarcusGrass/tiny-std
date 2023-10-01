use crate::io::{Read, Write};
use crate::net::{UnixListener, UnixStream};
use crate::unix::fd::AsRawFd;
use rusl::platform::{PollEvents, PollFd};
use rusl::string::unix_str::UnixStr;

#[test]
fn test_ping_pong() {
    let sock_path = UnixStr::try_from_str("/tmp/test-sock/sock1\0").unwrap();
    let _ = crate::fs::remove_file(sock_path);
    let _ = crate::fs::create_dir(UnixStr::try_from_str("/tmp/test-sock\0").unwrap());
    let listener = UnixListener::bind(sock_path, false).unwrap();
    let mut client = UnixStream::connect(sock_path, false).unwrap();
    let mut client_handle = listener.accept(true).unwrap();
    let write_in = &[8, 8, 8, 8];
    client_handle.write_all(write_in).unwrap();
    client_handle.flush().unwrap();
    rusl::select::ppoll(
        &mut [PollFd::new(client.as_raw_fd(), PollEvents::POLLOUT)],
        None,
        None,
    )
    .unwrap();
    let mut dest = [0u8; 4];
    client.read_exact(&mut dest).unwrap();
    assert_eq!(write_in, &dest);
}
