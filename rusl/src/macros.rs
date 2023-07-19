#[macro_export]
macro_rules! bail_on_below_zero {
    ($res: expr, $out_line: expr) => {
        if $crate::platform::is_syscall_error($res) {
            // Flip the errno
            return Err($crate::Error::with_code($out_line, 0 - $res as i32));
        }
    };
}

#[macro_export]
macro_rules! transparent_bitflags {
    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty {
            const DEFAULT = $const_default:expr;
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:ident = $value:expr;
            )*
        }
    ) => {
        $(#[$outer])*
        #[repr(transparent)]
        #[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
        $vis struct $BitFlags($T);

        impl $BitFlags {

            const DEFAULT: Self = Self($const_default);
            $(
                $(#[$inner $($args)*])*
                $vis const $Flag: Self = Self($value);
            )*

            #[must_use]
            #[inline(always)]
            #[allow(dead_code)]
            $vis const fn bits(&self) -> $T {
                self.0
            }

            #[inline]
            #[must_use]
            #[allow(dead_code)]
            $vis const fn empty() -> Self {
                Self::DEFAULT
            }

            #[inline]
            #[must_use]
            #[allow(dead_code)]
            $vis fn contains(&self, other: Self) -> bool {
                self.0 & other.0 != Self::DEFAULT.0
            }
        }

        impl Default for $BitFlags {
            #[inline]
            fn default() -> Self {
                Self::empty()
            }
        }
        impl core::fmt::Debug for $BitFlags {
            #[allow(unreachable_patterns)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match *self {
                    $(
                    Self::$Flag => f.write_fmt(core::format_args!("{}: {} = {}", stringify!($BitFlags), stringify!($Flag), self.0)),
                    )*
                    _ => f.write_fmt(core::format_args!("{}: UNKNOWN = {}", stringify!($BitFlags), self.0))
                }
            }
        }

        impl core::ops::BitOr for $BitFlags {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitAnd for $BitFlags {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitAndAssign for $BitFlags {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl core::ops::BitOrAssign for $BitFlags {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl From<$T> for $BitFlags {
            #[inline]
            fn from(val: $T) -> Self {
                Self(val)
            }
        }

        impl From<$BitFlags> for $T {
            #[inline]
            fn from(val: $BitFlags) -> $T {
                val.0
            }
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! errno_or_throw {
    ($res: expr) => {
        match $res {
            Ok(_) => panic!("Expected error, found success!"),
            Err(e) => e.code.unwrap(),
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! expect_errno {
    ($errno: expr, $res: expr) => {
        assert_eq!($errno, $crate::errno_or_throw!($res));
    };
}
