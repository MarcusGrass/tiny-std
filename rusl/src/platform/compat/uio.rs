use core::marker::PhantomData;

use linux_rust_bindings::uio::iovec;

#[repr(transparent)]
pub struct IoSlice<'a> {
    #[allow(dead_code)]
    vec: iovec,
    _p: PhantomData<&'a [u8]>,
}

impl<'a> IoSlice<'a> {
    #[must_use]
    pub fn new(buf: &[u8]) -> Self {
        Self {
            vec: iovec {
                iov_base: buf.as_ptr().cast_mut().cast(),
                iov_len: buf.len() as u64,
            },
            _p: PhantomData::default(),
        }
    }
}

#[repr(transparent)]
pub struct IoSliceMut<'a> {
    #[allow(dead_code)]
    pub(crate) vec: iovec,
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
            _p: PhantomData::default(),
        }
    }
}
