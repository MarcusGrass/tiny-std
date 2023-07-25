use crate::platform::is_syscall_error;
use crate::Error;
use core::fmt::Formatter;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NonNegativeI32(pub(crate) i32);

impl core::fmt::Display for NonNegativeI32 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for NonNegativeI32 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl NonNegativeI32 {
    pub const MAX: Self = Self(i32::MAX);
    pub const ZERO: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }

    /// Returns a `NonNegativeI32` if the supplied i32 is non-negative.
    /// # Errors
    /// Returns the supplied value
    #[inline]
    pub const fn try_new(val: i32) -> Result<Self, i32> {
        if val >= 0 {
            Ok(Self(val))
        } else {
            Err(val)
        }
    }

    /// Const constructor which panics on error, useful for constructing constants, as
    /// the compilation will make sure this is correct and safe.
    /// If used outside of a const context however, this will just panic on failure, which
    /// sucks, if not in const, then run `try_new`.
    /// # Panics
    /// Only when constructing a negative i32, don't use this function outside of a const context.
    #[inline]
    #[must_use]
    pub const fn comptime_checked_new(val: i32) -> Self {
        assert!(
            val >= 0,
            "Tried to comptime create a NonNegativeI32 from a value less than zero"
        );
        Self(val)
    }

    #[inline]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub(crate) const fn coerce_from_register(
        val: usize,
        err_msg: &'static str,
    ) -> Result<Self, Error> {
        if is_syscall_error(val) {
            let res = val as i32;
            Err(Error::with_code(err_msg, 0 - res))
        } else {
            Ok(Self(val as i32))
        }
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn into_u32(self) -> u32 {
        self.0 as u32
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn into_u64(self) -> u64 {
        self.0 as u64
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn into_u128(self) -> u128 {
        self.0 as u128
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn into_usize(self) -> usize {
        self.0 as usize
    }
}

impl BitAnd for NonNegativeI32 {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.bitand(rhs.0))
    }
}

impl BitAndAssign for NonNegativeI32 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0.bitand_assign(rhs.0);
    }
}

impl BitOr for NonNegativeI32 {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bitor(rhs.0))
    }
}

impl BitOrAssign for NonNegativeI32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0.bitor_assign(rhs.0);
    }
}
