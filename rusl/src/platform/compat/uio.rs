use core::marker::PhantomData;

pub use linux_rust_bindings::uio::iovec;

#[repr(transparent)]
pub struct IoSlice<'a> {
    #[allow(dead_code)]
    pub vec: iovec,
    _p: PhantomData<&'a [u8]>,
}

impl<'a> IoSlice<'a> {
    #[must_use]
    pub const fn new(buf: &[u8]) -> Self {
        Self {
            vec: iovec {
                iov_base: buf.as_ptr().cast_mut().cast(),
                iov_len: buf.len() as u64,
            },
            _p: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct IoSliceMut<'a> {
    pub vec: iovec,
    _p: PhantomData<&'a mut [u8]>,
}

impl<'a> IoSliceMut<'a> {
    #[must_use]
    pub fn new(buf: &mut [u8]) -> Self {
        Self {
            vec: iovec {
                iov_base: buf.as_mut_ptr().cast(),
                iov_len: buf.len() as u64,
            },
            _p: PhantomData,
        }
    }
}
