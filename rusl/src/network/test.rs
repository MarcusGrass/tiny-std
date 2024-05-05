use crate::error::Errno;
use crate::network::{get_inet_sock_name, get_unix_sock_name};
use crate::platform::{
    AddressFamily, ControlMessageSend, IoSlice, IoSliceMut, MsgHdrBorrow, NonNegativeI32,
    OpenFlags, PollEvents, PollFd, SocketAddressInet, SocketAddressUnix, SocketFlags,
    SocketOptions, SocketType,
};
use crate::unistd::{close, open, unlink};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

#[test]
fn get_dynamic_inet_sock_name() {
    let srv_sock = super::socket(
        AddressFamily::AF_INET,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        6,
    )
    .unwrap();
    let addr = SocketAddressInet::new([0, 0, 0, 0], 0);
    super::bind_inet(srv_sock, &addr).unwrap();
    // Dynamic port should have changed
    let assigned = get_inet_sock_name(srv_sock).unwrap();
    assert_ne!(addr.0.sin_port, assigned.0.sin_port);
    let _ = close(srv_sock);
}

#[test]
fn read_unix_sock_name() {
    let addr_raw = unix_lit!("test-files/socket/unix-sockname-test");
    let _ = unlink(addr_raw);
    let srv_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    let addr = SocketAddressUnix::try_from_unix(addr_raw).unwrap();
    super::bind_unix(srv_sock, &addr).unwrap();
    // Should be the same path as the socket address supplied to bind
    let assigned = get_unix_sock_name(srv_sock).unwrap();
    assert_eq!(addr.addr.0.sun_path, assigned.addr.0.sun_path);
    assert_eq!(addr.addr.0.sun_family, assigned.addr.0.sun_family);
    assert_eq!(addr.addr_len, assigned.addr_len);
    let _ = close(srv_sock);
}

#[test]
fn send_recv_msg() {
    const FIFTEEN: NonNegativeI32 = NonNegativeI32::comptime_checked_new(15);
    let addr_raw = unix_lit!("test-files/socket/tmp-recvmsg");
    let _ = unlink(addr_raw);
    let srv_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    let addr = SocketAddressUnix::try_from_unix(addr_raw).unwrap();
    super::bind_unix(srv_sock, &addr).unwrap();
    let cl_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    //let server_to_client_con = std::sync::Arc::new(std::sync::Mutex::new(None));
    //let stc_clone = server_to_client_con.clone();
    let listening = std::sync::Arc::new(AtomicBool::new(false));
    let lc = listening.clone();
    let listen_thread = std::thread::spawn(move || {
        super::listen(srv_sock, FIFTEEN).unwrap();
        lc.store(true, Ordering::SeqCst);
        let client = super::accept_unix(srv_sock, SocketFlags::SOCK_CLOEXEC)
            .unwrap()
            .0;
        let mut poll_fds = [PollFd::new(client, PollEvents::POLLIN)];
        crate::select::ppoll(&mut poll_fds, None, None).unwrap();
        assert_eq!(PollEvents::POLLIN, poll_fds[0].received_events());
        let mut space = [0u8; 64];
        let io = &mut [IoSliceMut::new(&mut space)];
        let mut ctrl_space = [0u8; 64];
        let mut hdr = MsgHdrBorrow::create_recv(io, Some(&mut ctrl_space));
        let re = super::recvmsg(client, &mut hdr, 0).unwrap();
        assert_eq!(5, re);
        assert_eq!(b"Hello", &space[..5]);
        let mut ctrl = hdr.control_messages();
        assert!(ctrl.next().is_none());
    });
    while !listening.load(Ordering::SeqCst) {}
    super::connect_unix(cl_sock, &addr).unwrap();
    let io_out = &[IoSlice::new(b"Hello")];
    let snd = MsgHdrBorrow::create_send(None, io_out, None);
    let send = super::sendmsg(cl_sock, &snd, 0).unwrap();
    assert_eq!(5, send);
    listen_thread.join().unwrap();
}
#[test]
fn send_recv_msg_with_control_single() {
    const FIFTEEN: NonNegativeI32 = NonNegativeI32::comptime_checked_new(15);
    let addr_raw = unix_lit!("test-files/socket/tmp-recvmsg-crtl");
    let _ = unlink(addr_raw);
    let srv_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    let addr = SocketAddressUnix::try_from_unix(addr_raw).unwrap();
    super::bind_unix(srv_sock, &addr).unwrap();
    let cl_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    //let server_to_client_con = std::sync::Arc::new(std::sync::Mutex::new(None));
    //let stc_clone = server_to_client_con.clone();
    let listening = std::sync::Arc::new(AtomicBool::new(false));
    let lc = listening.clone();
    let fd1 = open(unix_lit!("/proc/mounts"), OpenFlags::O_RDONLY).unwrap();
    let fds = [fd1];
    let listen_thread = std::thread::spawn(move || {
        super::listen(srv_sock, FIFTEEN).unwrap();
        lc.store(true, Ordering::SeqCst);
        let client = super::accept_unix(srv_sock, SocketFlags::SOCK_CLOEXEC)
            .unwrap()
            .0;
        let mut poll_fds = [PollFd::new(client, PollEvents::POLLIN)];
        crate::select::ppoll(&mut poll_fds, None, None).unwrap();
        assert_eq!(PollEvents::POLLIN, poll_fds[0].received_events());
        let mut space = [0u8; 64];
        let io = &mut [IoSliceMut::new(&mut space)];
        let mut ctrl_space = [0u8; 64];
        let mut hdr = MsgHdrBorrow::create_recv(io, Some(&mut ctrl_space));
        let re = super::recvmsg(client, &mut hdr, 0).unwrap();
        assert_eq!(5, re);
        assert_eq!(b"Hello", &space[..5]);
        let mut ctrl = hdr.control_messages();
        let scm_next = ctrl.next().unwrap();
        match scm_next {
            ControlMessageSend::ScmRights(recv) => {
                // Sending the fds over is in this case, since it's the same process, equivalent to a dup-call. It's the same file but different fds.
                // If it would have been the exact same, this would be a failure, just data serialization not actual fd-passing.
                assert_eq!(1, recv.len());
                assert!(recv[0] > fds[0]);
            }
        }
        assert!(ctrl.next().is_none());
    });
    while !listening.load(Ordering::SeqCst) {}
    super::connect_unix(cl_sock, &addr).unwrap();
    let io_out = &[IoSlice::new(b"Hello")];
    let snd = MsgHdrBorrow::create_send(None, io_out, Some(ControlMessageSend::ScmRights(&fds)));
    let send = super::sendmsg(cl_sock, &snd, 0).unwrap();
    assert_eq!(5, send);
    listen_thread.join().unwrap();
}

#[test]
fn send_recv_msg_with_control_multi() {
    const FIFTEEN: NonNegativeI32 = NonNegativeI32::comptime_checked_new(15);
    let addr_raw = unix_lit!("test-files/socket/tmp-recvmsg-crtl");
    let _ = unlink(addr_raw);
    let srv_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    let addr = SocketAddressUnix::try_from_unix(addr_raw).unwrap();
    super::bind_unix(srv_sock, &addr).unwrap();
    let cl_sock = super::socket(
        AddressFamily::AF_UNIX,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        0,
    )
    .unwrap();
    //let server_to_client_con = std::sync::Arc::new(std::sync::Mutex::new(None));
    //let stc_clone = server_to_client_con.clone();
    let listening = std::sync::Arc::new(AtomicBool::new(false));
    let lc = listening.clone();
    let fd1 = open(unix_lit!("/proc/mounts"), OpenFlags::O_RDONLY).unwrap();
    let fd2 = open(unix_lit!("/proc/mounts"), OpenFlags::O_RDONLY).unwrap();
    let fds = [fd1, fd2];
    let listen_thread = std::thread::spawn(move || {
        super::listen(srv_sock, FIFTEEN).unwrap();
        lc.store(true, Ordering::SeqCst);
        let client = super::accept_unix(srv_sock, SocketFlags::SOCK_CLOEXEC)
            .unwrap()
            .0;
        let mut poll_fds = [PollFd::new(client, PollEvents::POLLIN)];
        crate::select::ppoll(&mut poll_fds, None, None).unwrap();
        assert_eq!(PollEvents::POLLIN, poll_fds[0].received_events());
        let mut space = [0u8; 64];
        let io = &mut [IoSliceMut::new(&mut space)];
        let mut ctrl_space = [0u8; 64];
        let mut hdr = MsgHdrBorrow::create_recv(io, Some(&mut ctrl_space));
        let re = super::recvmsg(client, &mut hdr, 0).unwrap();
        assert_eq!(5, re);
        assert_eq!(b"Hello", &space[..5]);
        let mut ctrl = hdr.control_messages();
        let scm_next = ctrl.next().unwrap();
        match scm_next {
            ControlMessageSend::ScmRights(recv) => {
                // Sending the fds over is in this case, since it's the same process, equivalent to a dup-call. It's the same file but different fds.
                // If it would have been the exact same, this would be a failure, just data serialization not actual fd-passing.
                assert_eq!(2, recv.len());
                assert!(recv[1] > recv[0]);
                assert!(recv[0] > fds[1]);
            }
        }
        assert!(ctrl.next().is_none());
    });
    while !listening.load(Ordering::SeqCst) {}
    super::connect_unix(cl_sock, &addr).unwrap();
    let io_out = &[IoSlice::new(b"Hello")];
    let snd = MsgHdrBorrow::create_send(None, io_out, Some(ControlMessageSend::ScmRights(&fds)));
    let send = super::sendmsg(cl_sock, &snd, 0).unwrap();
    assert_eq!(5, send);
    listen_thread.join().unwrap();
}

#[test]
fn test_tcp() {
    const FIFTEEN: NonNegativeI32 = NonNegativeI32::comptime_checked_new(15);
    const EXPECT_REQ: &[u8] = b"Hello server!";
    const EXPECT_RES: &[u8] = b"Hello client!";
    let srv_sock = super::socket(
        AddressFamily::AF_INET,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        6,
    )
    .unwrap();
    let mut addr = SocketAddressInet::new([0, 0, 0, 0], 0);
    super::bind_inet(srv_sock, &addr).unwrap();
    // Get dynamically assigned port
    addr.0.sin_port = get_inet_sock_name(srv_sock).unwrap().0.sin_port;
    super::listen(srv_sock, FIFTEEN).unwrap();
    let _srv = std::thread::spawn(move || {
        let (fd, _addr) = super::accept_inet(srv_sock, SocketFlags::SOCK_CLOEXEC).unwrap();
        let mut buf = [0u8; EXPECT_REQ.len()];
        let mut read = 0;
        while read < EXPECT_REQ.len() {
            let count = crate::unistd::read(fd, &mut buf);
            match count {
                Ok(bytes) => {
                    read += bytes;
                }
                Err(e) => {
                    if e.code.unwrap() == Errno::EAGAIN {
                        continue;
                    }
                    panic!("Error reading {e}");
                }
            }
        }
        assert_eq!(EXPECT_REQ, &buf);
        let written = crate::unistd::write(fd, EXPECT_RES).unwrap();
        assert_eq!(EXPECT_RES.len(), written);
    });
    std::thread::sleep(Duration::from_millis(10));
    let clnt = super::socket(
        AddressFamily::AF_INET,
        SocketOptions::new(SocketType::SOCK_STREAM, SocketFlags::SOCK_CLOEXEC),
        6,
    )
    .unwrap();
    super::connect_inet(clnt, &addr).unwrap();
    assert_eq!(
        EXPECT_REQ.len(),
        crate::unistd::write(clnt, EXPECT_REQ).unwrap()
    );
    let mut buf = [0u8; EXPECT_RES.len()];
    assert_eq!(
        EXPECT_RES.len(),
        crate::unistd::read(clnt, &mut buf).unwrap()
    );
    assert_eq!(EXPECT_RES, buf);
}
