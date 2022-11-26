use core::marker::PhantomData;

pub type RawFd = i32;

#[repr(transparent)]
#[derive(Debug)]
pub struct OwnedFd(pub(crate) RawFd);

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
