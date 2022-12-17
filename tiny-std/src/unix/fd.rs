use core::marker::PhantomData;
use rusl::platform::OpenFlags;
use rusl::unistd::fcntl_set_file_status;

pub type RawFd = i32;

#[repr(transparent)]
#[derive(Debug)]
pub struct OwnedFd(pub(crate) RawFd);

impl OwnedFd {
    pub const unsafe fn from_raw(raw: RawFd) -> Self {
        Self(raw)
    }

    pub fn set_nonblocking(&self) -> crate::error::Result<()> {
        let orig = rusl::unistd::fcntl_get_file_status(self.0, OpenFlags::empty())?;
        fcntl_set_file_status(self.0, orig | OpenFlags::O_NONBLOCK)?;
        Ok(())
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
