use core::marker::PhantomData;
use rusl::platform::OpenFlags;
use rusl::unistd::fcntl_set_file_status;

pub type RawFd = i32;

#[repr(transparent)]
#[derive(Debug)]
pub struct OwnedFd(pub(crate) RawFd);

impl OwnedFd {
    /// Create an `OwnedFd` from a `RawFd`
    /// # Safety
    /// `fd` is valid and not used elsewhere, see `File::from_raw_fd`
    #[must_use]
    pub const unsafe fn from_raw(raw: RawFd) -> Self {
        Self(raw)
    }

    /// Sets this owned FD as non-blocking
    /// # Errors
    /// This FD is invalid, through unsafe creation
    #[inline]
    pub fn set_nonblocking(&self) -> crate::error::Result<()> {
        set_fd_nonblocking(self.as_raw_fd())
    }
}

impl AsRawFd for OwnedFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Drop for OwnedFd {
    fn drop(&mut self) {
        // Best attempt
        let _ = rusl::unistd::close(self.0);
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct BorrowedFd<'fd> {
    pub(crate) fd: RawFd,
    _pd: PhantomData<&'fd OwnedFd>,
}

impl<'a> BorrowedFd<'a> {
    pub(crate) fn new(fd: RawFd) -> Self {
        Self {
            fd,
            _pd: PhantomData::default(),
        }
    }
}

impl<'a> AsRawFd for BorrowedFd<'a> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

pub trait AsRawFd {
    fn as_raw_fd(&self) -> RawFd;
}

#[inline]
pub(crate) fn set_fd_nonblocking(raw_fd: RawFd) -> crate::error::Result<()> {
    let orig = rusl::unistd::fcntl_get_file_status(raw_fd)?;
    fcntl_set_file_status(raw_fd, orig | OpenFlags::O_NONBLOCK)?;
    Ok(())
}
