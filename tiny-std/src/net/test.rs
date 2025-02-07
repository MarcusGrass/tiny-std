use core::time::Duration;

use crate::io::{Read, Write};
use crate::net::{
    Ip, SocketAddress, TcpListener, TcpStream, TcpTryConnect, UnixListener, UnixStream,
};
use crate::time::MonotonicInstant;
use crate::unix::fd::AsRawFd;
use rusl::error::Errno;
use rusl::platform::{PollEvents, PollFd};
use rusl::string::unix_str::UnixStr;

#[test]
fn test_unix_ping_pong() {
    let sock_path = UnixStr::try_from_str("/tmp/test-sock/sock1\0").unwrap();
    let _ = crate::fs::remove_file(sock_path);
    crate::fs::create_dir_all(UnixStr::try_from_str("/tmp/test-sock/\0").unwrap()).unwrap();
    let mut listener = UnixListener::bind(sock_path).unwrap();
    assert!(matches!(listener.try_accept(), Ok(None)));
    let client = UnixStream::connect(sock_path).unwrap();
    let client_handle = listener.accept().unwrap();
    verify_communication(client, client_handle);
}

#[test]
fn test_unix_accept_timeout() {
    let sock_path = UnixStr::try_from_str("/tmp/test-sock/sock2\0").unwrap();
    let _ = crate::fs::remove_file(sock_path);
    crate::fs::create_dir_all(UnixStr::try_from_str("/tmp/test-sock/\0").unwrap()).unwrap();
    let mut listener = UnixListener::bind(sock_path).unwrap();
    assert!(
        matches!(
            listener.accept_with_timeout(core::time::Duration::from_millis(15)),
            Err(crate::Error::Timeout)
        ),
        "Expected timeout"
    );
}

#[test]
fn test_unix_try_accept() {
    let sock_path = UnixStr::try_from_str("/tmp/test-sock/sock3\0").unwrap();
    let _ = crate::fs::remove_file(sock_path);
    crate::fs::create_dir_all(UnixStr::try_from_str("/tmp/test-sock/\0").unwrap()).unwrap();
    let mut listener = UnixListener::bind(sock_path).unwrap();
    assert!(matches!(listener.try_accept(), Ok(None)));
    let client = UnixStream::connect(sock_path).unwrap();
    let client_handle = listener.try_accept().unwrap().unwrap();
    verify_communication(client, client_handle);
}

#[test]
fn test_unix_try_connect() {
    let sock_path = UnixStr::try_from_str("/tmp/test-sock/sock4\0").unwrap();
    let _ = crate::fs::remove_file(sock_path);
    crate::fs::create_dir_all(UnixStr::try_from_str("/tmp/test-sock/\0").unwrap()).unwrap();
    let mut listener = UnixListener::bind(sock_path).unwrap();
    assert!(matches!(listener.try_accept(), Ok(None)));
    let client = UnixStream::try_connect(sock_path).unwrap().unwrap();
    let client_handle = listener.accept().unwrap();
    verify_communication(client, client_handle);
}

fn verify_communication<C: Read + Write + AsRawFd, H: Read + Write + AsRawFd>(
    mut client: C,
    mut client_handle: H,
) {
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

#[test]
fn test_tcp_ping_pong() {
    let ip = Ip::V4([127, 0, 0, 1]);
    let mut listener = TcpListener::bind(&SocketAddress::new(ip, 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let client = TcpStream::connect(&addr).unwrap();
    let client_handle = listener.accept().unwrap();
    verify_communication(client, client_handle);
}

#[test]
fn test_tcp_accept_timeout() {
    let ip = Ip::V4([127, 0, 0, 1]);
    let mut listener = TcpListener::bind(&SocketAddress::new(ip, 0)).unwrap();
    assert!(
        matches!(
            listener.accept_with_timeout(core::time::Duration::from_millis(15)),
            Err(crate::Error::Timeout)
        ),
        "Expected timeout"
    );
}

#[test]
fn test_tcp_try_accept() {
    let ip = Ip::V4([127, 0, 0, 1]);
    let mut listener = TcpListener::bind(&SocketAddress::new(ip, 0)).unwrap();
    assert!(matches!(listener.try_accept(), Ok(None)));
    let addr = listener.local_addr().unwrap();
    let client = TcpStream::connect(&addr).unwrap();
    let client_handle = listener.try_accept().unwrap().unwrap();
    verify_communication(client, client_handle);
}

#[test]
fn test_tcp_try_connect() {
    let ip = Ip::V4([127, 0, 0, 1]);
    let TcpTryConnect::InProgress(bad_address_con) =
        TcpStream::try_connect(&SocketAddress::new(ip, u16::MAX - 1)).unwrap()
    else {
        panic!("Failed to start connection");
    };
    let Err(e) = bad_address_con.try_connect() else {
        panic!("Expected err on invalid port, some chance of spurious error due to port being used by someone else");
    };
    assert!(matches!(
        e,
        crate::error::Error::Os {
            msg: _,
            code: Errno::ECONNREFUSED
        }
    ));
    let mut listener = TcpListener::bind(&SocketAddress::new(ip, 0)).unwrap();
    assert!(matches!(listener.try_accept(), Ok(None)));
    let addr = listener.local_addr().unwrap();
    let TcpTryConnect::InProgress(con) = TcpStream::try_connect(&addr).unwrap() else {
        panic!("Failed to start connection");
    };
    let jh = std::thread::spawn(move || {
        let start = MonotonicInstant::now();
        let mut con = con;
        loop {
            match con.try_connect() {
                Ok(TcpTryConnect::Connected(c)) => break c,
                Ok(TcpTryConnect::InProgress(c)) => con = c,
                Err(e) => panic!("failed try_connect: {e}"),
            }
            assert!(
                (start.elapsed() <= Duration::from_millis(15)),
                "Failed to connect TCP in time"
            );
        }
    });
    let client_handle = listener
        .accept_with_timeout(Duration::from_millis(15))
        .unwrap();
    let client = jh.join().unwrap();
    verify_communication(client, client_handle);
}

#[test]
fn test_tcp_try_connect_then_block() {
    let ip = Ip::V4([127, 0, 0, 1]);
    let mut listener = TcpListener::bind(&SocketAddress::new(ip, 0)).unwrap();
    assert!(matches!(listener.try_accept(), Ok(None)));
    let addr = listener.local_addr().unwrap();
    let TcpTryConnect::InProgress(con) = TcpStream::try_connect(&addr).unwrap() else {
        panic!("Failed to start connection");
    };
    let jh = std::thread::spawn(move || con.connect_blocking().unwrap());
    let client_handle = listener
        .accept_with_timeout(Duration::from_millis(15))
        .unwrap();
    let client = jh.join().unwrap();
    verify_communication(client, client_handle);
}
