use core::time::Duration;

use rusl::error::Errno;
use rusl::network::get_inet_sock_name;
use rusl::platform::{
    AddressFamily, NonNegativeI32, PollEvents, SocketAddressInet, SocketAddressUnix, SocketFlags,
    SocketOptions, SocketType,
};
use rusl::string::unix_str::UnixStr;

use crate::error::Result;
use crate::io::{Read, Write};
use crate::sock::{
    blocking_read_nonblock_sock, blocking_write_nonblock_sock, sock_nonblock_op_poll_if_not_ready,
};
use crate::unix::fd::{AsRawFd, OwnedFd, RawFd};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct UnixStream(OwnedFd);

impl UnixStream {
    /// Creates and connects a non-blocking `UnixStream` at the specified path, blocking during the
    /// connection attempt
    /// # Errors
    /// Various OS errors relating to permissions, and missing paths
    #[inline]
    pub fn connect(path: &UnixStr) -> Result<Self> {
        Self::do_connect(path, None)
    }

    fn do_connect(path: &UnixStr, timeout: Option<Duration>) -> Result<Self> {
        let fd = rusl::network::socket(
            AddressFamily::AF_UNIX,
            SocketOptions::new(
                SocketType::SOCK_STREAM,
                SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
            ),
            0,
        )?;
        let addr = SocketAddressUnix::try_from_unix(path)?;
        if let Err(e) = sock_nonblock_op_poll_if_not_ready(
            fd,
            Errno::EAGAIN,
            PollEvents::POLLOUT,
            timeout,
            |sock| rusl::network::connect_unix(sock, &addr),
        ) {
            let _ = rusl::unistd::close(fd);
            return Err(e);
        }
        Ok(Self(OwnedFd(fd)))
    }

    /// Attempts to connect immediately without blocking, returns `Some` if successful, `None`
    /// otherwise
    /// # Errors
    /// Various OS errors relating to permissions, and missing paths
    pub fn try_connect(path: &UnixStr) -> Result<Option<Self>> {
        let fd = rusl::network::socket(
            AddressFamily::AF_UNIX,
            SocketOptions::new(
                SocketType::SOCK_STREAM,
                SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
            ),
            0,
        )?;
        let addr = SocketAddressUnix::try_from_unix(path)?;
        match rusl::network::connect_unix(fd, &addr) {
            Ok(()) => {}
            Err(e) if e.code == Some(Errno::EAGAIN) => {
                let _ = rusl::unistd::close(fd);
                return Ok(None);
            }
            Err(e) => {
                let _ = rusl::unistd::close(fd);
                return Err(e.into());
            }
        }
        Ok(Some(Self(OwnedFd(fd))))
    }
}

impl AsRawFd for UnixStream {
    fn as_raw_fd(&self) -> RawFd {
        self.0 .0
    }
}

impl Read for UnixStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        blocking_read_nonblock_sock(self.0 .0, buf, None)
    }
}

impl Write for UnixStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        blocking_write_nonblock_sock(self.0 .0, buf, None)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct UnixListener(OwnedFd);

impl UnixListener {
    /// Creates and binds a non-blocking `UnixListener` at the specified path
    /// Use the `blocking` variable to set as blocking or non-blocking
    /// # Errors
    /// Various OS errors relating to permissions, and missing paths
    pub fn bind(path: &UnixStr) -> Result<Self> {
        let fd = rusl::network::socket(
            AddressFamily::AF_UNIX,
            SocketOptions::new(
                SocketType::SOCK_STREAM,
                SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
            ),
            0,
        )?;
        let addr = SocketAddressUnix::try_from_unix(path)?;
        if let Err(e) = rusl::network::bind_unix(fd, &addr) {
            let _ = rusl::unistd::close(fd);
            return Err(e.into());
        }
        if let Err(e) = rusl::network::listen(fd, NonNegativeI32::MAX) {
            let _ = rusl::unistd::close(fd);
            return Err(e.into());
        }
        rusl::network::listen(fd, NonNegativeI32::MAX)?;
        Ok(Self(OwnedFd(fd)))
    }

    /// Accepts a new connection, `UnixListener`, blocking until it arrives
    /// # Errors
    /// Various OS errors relating to socket communication
    #[inline]
    pub fn accept(&mut self) -> Result<UnixStream> {
        self.do_accept(None)
    }

    /// Accepts a new connection, blocking until the specified `timeout`
    /// # Errors
    /// Various OS errors relating to socket communication
    #[inline]
    pub fn accept_with_timeout(&mut self, timeout: Duration) -> Result<UnixStream> {
        self.do_accept(Some(timeout))
    }

    /// Attempts to accept a new connection, returns `Some` if one is immediately available, `None`
    /// otherwise.
    /// # Errors
    /// Various OS errors relating to socket communication
    #[inline]
    pub fn try_accept(&mut self) -> Result<Option<UnixStream>> {
        let fd = match rusl::network::accept_unix(
            self.0 .0,
            SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
        ) {
            Ok((fd, _addr)) => fd,
            Err(e) if e.code == Some(Errno::EAGAIN) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        Ok(Some(UnixStream(OwnedFd(fd))))
    }

    fn do_accept(&mut self, timeout: Option<Duration>) -> Result<UnixStream> {
        let (fd, _addr) = sock_nonblock_op_poll_if_not_ready(
            self.0 .0,
            Errno::EAGAIN,
            PollEvents::POLLIN,
            timeout,
            |sock| {
                rusl::network::accept_unix(
                    sock,
                    SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
                )
            },
        )?;

        Ok(UnixStream(OwnedFd(fd)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SocketAddress {
    ip: Ip,
    port: u16,
}

impl SocketAddress {
    #[must_use]
    pub fn new(ip: Ip, port: u16) -> Self {
        Self { ip, port }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum Ip {
    /// 4 bytes representing the address, i.e. `127.0.0.1`
    V4([u8; 4]),
}

#[derive(Debug)]
pub struct TcpStream(OwnedFd);

#[derive(Debug)]
pub enum TcpTryConnect {
    Connected(TcpStream),
    InProgress(TcpStreamInProgress),
}

#[derive(Debug)]
pub struct TcpStreamInProgress(OwnedFd, SocketAddressInet);

impl TcpStreamInProgress {
    /// Try to continue establishing the connection started on this stream
    /// # Errors
    /// Connection failures
    pub fn try_connect(self) -> Result<TcpTryConnect> {
        match rusl::network::connect_inet(self.0 .0, &self.1) {
            Ok(()) => {}
            Err(e) if matches!(e.code, Some(Errno::EINPROGRESS)) => {
                return Ok(TcpTryConnect::InProgress(self));
            }
            Err(e) => {
                return Err(e.into());
            }
        }
        let Self(o, _s) = self;
        Ok(TcpTryConnect::Connected(TcpStream(o)))
    }

    /// Block on this in-progress connection until it succeeds or fails
    /// # Errors
    /// Connection failures
    pub fn connect_blocking(self) -> Result<TcpStream> {
        sock_nonblock_op_poll_if_not_ready(
            self.0 .0,
            Errno::EINPROGRESS,
            PollEvents::POLLOUT,
            None,
            |sock| rusl::network::connect_inet(sock, &self.1),
        )?;
        let Self(o, _addr) = self;
        Ok(TcpStream(o))
    }
}

impl TcpStream {
    /// Creates and connects a non-blocking [`TcpStream`] at the specified address, blocks until the
    /// connection is established.
    /// # Errors
    /// Various OS errors relating to permissions, and networking issues
    pub fn connect(addr: &SocketAddress) -> Result<Self> {
        Self::do_connect(addr, None)
    }

    /// Creates and connects a non-blocking [`TcpStream`] at the specified address with a connection
    /// timeout.
    /// # Errors
    /// Various OS errors relating to permissions, and networking issues
    pub fn connect_with_timeout(addr: &SocketAddress, timeout: Duration) -> Result<Self> {
        Self::do_connect(addr, Some(timeout))
    }

    /// Attempts to connect a non-blocking [`TcpStream`] at the specified address,
    /// returns `Some` if a connection can be established immediately and `None` if it can't.
    /// # Errors
    /// Various OS errors relating to permissions, and networking issues
    pub fn try_connect(addr: &SocketAddress) -> Result<TcpTryConnect> {
        let fd = rusl::network::socket(
            AddressFamily::AF_INET,
            SocketOptions::new(
                SocketType::SOCK_STREAM,
                SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
            ),
            6,
        )?;
        let addr = match addr.ip {
            Ip::V4(bytes) => SocketAddressInet::new(bytes, addr.port),
        };
        match rusl::network::connect_inet(fd, &addr) {
            Ok(()) => {}
            Err(e) if matches!(e.code, Some(Errno::EINPROGRESS)) => {
                return Ok(TcpTryConnect::InProgress(TcpStreamInProgress(
                    OwnedFd(fd),
                    addr,
                )));
            }
            Err(e) => {
                let _ = rusl::unistd::close(fd);
                return Err(e.into());
            }
        }
        Ok(TcpTryConnect::Connected(Self(OwnedFd(fd))))
    }

    #[expect(clippy::trivially_copy_pass_by_ref)]
    fn do_connect(addr: &SocketAddress, timeout: Option<Duration>) -> Result<Self> {
        let fd = rusl::network::socket(
            AddressFamily::AF_INET,
            SocketOptions::new(
                SocketType::SOCK_STREAM,
                SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
            ),
            6,
        )?;
        let addr = match addr.ip {
            Ip::V4(bytes) => SocketAddressInet::new(bytes, addr.port),
        };
        if let Err(e) = sock_nonblock_op_poll_if_not_ready(
            fd,
            Errno::EINPROGRESS,
            PollEvents::POLLOUT,
            timeout,
            |sock| rusl::network::connect_inet(sock, &addr),
        ) {
            let _ = rusl::unistd::close(fd);
            return Err(e);
        }
        Ok(Self(OwnedFd(fd)))
    }

    /// Reads from this socket, into the provided buffer, with the specified timeout
    /// # Errors
    /// Os-errors related to reads, or a timeout
    #[inline]
    pub fn read_with_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        blocking_read_nonblock_sock(self.0 .0, buf, Some(timeout))
    }
}

impl AsRawFd for TcpStream {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0 .0
    }
}

impl Read for TcpStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        blocking_read_nonblock_sock(self.0 .0, buf, None)
    }
}

impl Write for TcpStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        blocking_write_nonblock_sock(self.0 .0, buf, None)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct TcpListener(OwnedFd);

impl TcpListener {
    /// Create a socket, bind to it, and listen on it, creating a `TcpListener` at the provided address
    /// # Errors
    /// Various OS errors relating to permissions, and missing paths
    pub fn bind(addr: &SocketAddress) -> Result<Self> {
        let fd = rusl::network::socket(
            AddressFamily::AF_INET,
            SocketOptions::new(
                SocketType::SOCK_STREAM,
                SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
            ),
            6,
        )?;
        let addr = match addr.ip {
            Ip::V4(bytes) => SocketAddressInet::new(bytes, addr.port),
        };
        if let Err(e) = rusl::network::bind_inet(fd, &addr) {
            let _ = rusl::unistd::close(fd);
            return Err(e.into());
        }
        rusl::network::listen(fd, NonNegativeI32::MAX)?;
        Ok(Self(OwnedFd(fd)))
    }
    /// Get this socket's local bind address
    /// # Errors
    /// Various OS errors, most likely os out of resources
    pub fn local_addr(&self) -> Result<SocketAddress> {
        let name = get_inet_sock_name(self.0 .0)?;
        let (ip, port) = name.ipv4_addr();
        Ok(SocketAddress::new(Ip::V4(ip), port))
    }

    /// Attempt to accept a client connection on the socket
    /// # Errors
    /// Various OS-errors such as out of memory
    pub fn accept(&mut self) -> Result<TcpStream> {
        self.do_accept(None)
    }

    /// Attempt to accept a client connection on the socket with a timeout
    /// # Errors
    /// Various OS-errors such as out of memory, or a timeout
    pub fn accept_with_timeout(&mut self, timeout: Duration) -> Result<TcpStream> {
        self.do_accept(Some(timeout))
    }

    /// Attempt to accept a client connection on the socket, returns `Some` if one is immediately
    /// available, `None` otherwise
    /// # Errors
    /// Various OS-errors such as out of memory
    pub fn try_accept(&mut self) -> Result<Option<TcpStream>> {
        let fd = match rusl::network::accept_inet(
            self.0 .0,
            SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
        ) {
            Ok((fd, _addr)) => fd,
            Err(e) if e.code == Some(Errno::EAGAIN) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        Ok(Some(TcpStream(OwnedFd(fd))))
    }

    fn do_accept(&self, timeout: Option<Duration>) -> Result<TcpStream> {
        let (fd, _addr) = sock_nonblock_op_poll_if_not_ready(
            self.0 .0,
            Errno::EAGAIN,
            PollEvents::POLLIN,
            timeout,
            |sock| {
                rusl::network::accept_inet(
                    sock,
                    SocketFlags::SOCK_NONBLOCK | SocketFlags::SOCK_CLOEXEC,
                )
            },
        )?;

        Ok(TcpStream(OwnedFd(fd)))
    }
}
