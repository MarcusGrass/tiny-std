use crate::io::{Read, Write};
use crate::net::{UnixListener, UnixStream};

#[test]
fn test_ping_pong() {
    let sock_path = "/tmp/test-sock/sock1\0";
    let _ = crate::fs::remove_file(sock_path);
    let _ = crate::fs::create_dir("/tmp/test-sock\0");
    let listener = UnixListener::bind(sock_path, false).unwrap();
    let mut client = UnixStream::connect(sock_path, false).unwrap();
    let mut client_handle = listener.accept(true).unwrap();
    let write_in = &[8, 8, 8, 8];
    client_handle.write_all(write_in).unwrap();
    let mut dest = [0u8; 4];
    client.read_exact(&mut dest).unwrap();
    assert_eq!(write_in, &dest);
}
