use rusl::platform::{AddressFamily, SocketAddress, SocketType};
use rusl::string::unix_str::AsUnixStr;

use crate::error::Result;
use crate::io::{Read, Write};
use crate::unix::fd::{AsRawFd, OwnedFd, RawFd};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct UnixStream(OwnedFd);

impl UnixStream {
    /// Creates and connects a non-blocking `UnixStream` at the specified path
    /// # Errors
    /// Various OS errors relating to permissions, and missing paths
    pub fn connect<P: AsUnixStr>(path: P, blocking: bool) -> Result<Self> {
        let block = blocking
            .then(SocketType::empty)
            .unwrap_or(SocketType::SOCK_NONBLOCK);
        let fd = rusl::network::socket(AddressFamily::AF_UNIX, SocketType::SOCK_STREAM | block, 0)?;
        let addr = SocketAddress::try_from_unix(path)?;

        rusl::network::connect(fd, &addr)?;
        Ok(UnixStream(OwnedFd(fd)))
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
        Ok(rusl::unistd::read(self.0 .0, buf)?)
    }
}

impl Write for UnixStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(rusl::unistd::write(self.0 .0, buf)?)
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
    pub fn bind<P: AsUnixStr>(path: P, blocking: bool) -> Result<Self> {
        const BACKLOG: i32 = if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            -1
        } else {
            128
        };
        let block = blocking
            .then(SocketType::empty)
            .unwrap_or(SocketType::SOCK_NONBLOCK);
        let fd = rusl::network::socket(AddressFamily::AF_UNIX, SocketType::SOCK_STREAM | block, 0)?;
        let addr = SocketAddress::try_from_unix(path)?;
        rusl::network::bind(fd, &addr)?;
        rusl::network::listen(fd, BACKLOG)?;
        Ok(Self(OwnedFd(fd)))
    }

    /// Accepts a new connection, blocking if this `UnixListener` was previously set to be blocking
    /// Use the `blocking` variable to set the incoming `UnixStream` as blocking or non-blocking
    /// # Errors
    /// Various OS errors relating to socket communication
    /// `EAGAIN` if this listener is set to non-blocking and there are no ready connections
    pub fn accept(&self, blocking: bool) -> Result<UnixStream> {
        let block = blocking
            .then(SocketType::empty)
            .unwrap_or(SocketType::SOCK_NONBLOCK);
        let client = rusl::network::accept(self.0 .0, None, SocketType::SOCK_CLOEXEC | block)?;
        Ok(UnixStream(OwnedFd(client)))
    }
}
