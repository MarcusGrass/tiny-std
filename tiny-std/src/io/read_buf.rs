use core::cmp;
use core::mem::MaybeUninit;

/// A wrapper around a byte buffer that is incrementally filled and initialized.
///
/// This type is a sort of "double cursor". It tracks three regions in the buffer: a region at the beginning of the
/// buffer that has been logically filled with data, a region that has been initialized at some point but not yet
/// logically filled, and a region at the end that is fully uninitialized. The filled region is guaranteed to be a
/// subset of the initialized region.
///
/// In summary, the contents of the buffer can be visualized as:
/// ```not_rust
/// [             capacity              ]
/// [ filled |         unfilled         ]
/// [    initialized    | uninitialized ]
/// ```
pub(crate) struct ReadBuf<'a> {
    buf: &'a mut [MaybeUninit<u8>],
    filled: usize,
    initialized: usize,
}

impl<'a> ReadBuf<'a> {
    /// Creates a new `ReadBuf` from a fully uninitialized buffer.
    ///
    /// Use `assume_init` if part of the buffer is known to be already initialized.
    #[inline]
    pub(crate) fn uninit(buf: &'a mut [MaybeUninit<u8>]) -> ReadBuf<'a> {
        ReadBuf {
            buf,
            filled: 0,
            initialized: 0,
        }
    }

    /// Returns the total capacity of the buffer.
    #[inline]
    #[must_use]
    pub(crate) fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Returns a mutable reference to the initialized portion of the buffer.
    ///
    /// This includes the filled portion.
    #[inline]
    pub(crate) fn initialized_mut(&mut self) -> &mut [u8] {
        //SAFETY: We only slice the initialized part of the buffer, which is always valid
        unsafe { slice_assume_init_mut(&mut self.buf[0..self.initialized]) }
    }

    /// Returns a mutable reference to the uninitialized part of the buffer.
    ///
    /// It is safe to uninitialize any of these bytes.
    #[inline]
    pub(crate) fn uninitialized_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buf[self.initialized..]
    }

    /// Returns a mutable reference to the unfilled part of the buffer, ensuring it is fully initialized.
    ///
    /// Since `ReadBuf` tracks the region of the buffer that has been initialized, this is effectively "free" after
    /// the first use.
    #[inline]
    pub(crate) fn initialize_unfilled(&mut self) -> &mut [u8] {
        // should optimize out the assertion
        self.initialize_unfilled_to(self.remaining())
    }

    /// Returns a mutable reference to the first `n` bytes of the unfilled part of the buffer, ensuring it is
    /// fully initialized.
    ///
    /// # Panics
    ///
    /// Panics if `self.remaining()` is less than `n`.
    #[inline]
    pub(crate) fn initialize_unfilled_to(&mut self, n: usize) -> &mut [u8] {
        assert!(self.remaining() >= n);

        let extra_init = self.initialized - self.filled;
        // If we don't have enough initialized, do zeroing
        if n > extra_init {
            let uninit = n - extra_init;
            let unfilled = &mut self.uninitialized_mut()[0..uninit];

            for byte in &mut *unfilled {
                byte.write(0);
            }

            // SAFETY: we just initialized uninit bytes, and the previous bytes were already init
            unsafe {
                self.assume_init(n);
            }
        }

        let filled = self.filled;

        &mut self.initialized_mut()[filled..filled + n]
    }

    /// Returns the number of bytes at the end of the slice that have not yet been filled.
    #[inline]
    #[must_use]
    pub(crate) fn remaining(&self) -> usize {
        self.capacity() - self.filled
    }

    /// Increases the size of the filled region of the buffer.
    ///
    /// The number of initialized bytes is not changed.
    ///
    /// # Panics
    ///
    /// Panics if the filled region of the buffer would become larger than the initialized region.
    #[inline]
    pub(crate) fn add_filled(&mut self, n: usize) {
        self.set_filled(self.filled + n);
    }

    /// Sets the size of the filled region of the buffer.
    ///
    /// The number of initialized bytes is not changed.
    ///
    /// Note that this can be used to *shrink* the filled region of the buffer in addition to growing it (for
    /// example, by a `Read` implementation that compresses data in-place).
    ///
    /// # Panics
    ///
    /// Panics if the filled region of the buffer would become larger than the initialized region.
    #[inline]
    pub(crate) fn set_filled(&mut self, n: usize) {
        assert!(n <= self.initialized);
        self.filled = n;
    }

    /// Asserts that the first `n` unfilled bytes of the buffer are initialized.
    ///
    /// `ReadBuf` assumes that bytes are never de-initialized, so this method does nothing when called with fewer
    /// bytes than are already known to be initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the first `n` unfilled bytes of the buffer have already been initialized.
    #[inline]
    pub(crate) unsafe fn assume_init(&mut self, n: usize) {
        self.initialized = cmp::max(self.initialized, self.filled + n);
    }

    /// Returns the amount of bytes that have been filled.
    #[inline]
    #[must_use]
    pub(crate) fn filled_len(&self) -> usize {
        self.filled
    }

    /// Returns the amount of bytes that have been initialized.
    #[inline]
    #[must_use]
    pub(crate) fn initialized_len(&self) -> usize {
        self.initialized
    }
}

#[inline]
pub(crate) unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: casting `slice` to a `*const [T]` is safe since the caller guarantees that
    // `slice` is initialized, and `MaybeUninit` is guaranteed to have the same layout as `T`.
    // The pointer obtained is valid since it refers to memory owned by `slice` which is a
    // reference and thus guaranteed to be valid for reads.
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}
