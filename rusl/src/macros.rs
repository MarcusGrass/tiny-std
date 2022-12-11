#[macro_export]
macro_rules! bail_on_below_zero {
    ($res: expr, $out_line: expr) => {
        if $res > usize::MAX - 256 {
            // Flip the errno
            return Err($crate::Error::with_code($out_line, 0 - $res as i32));
        }
    };
}

#[macro_export]
macro_rules! const_errs {
    ($($num: expr, $name:ident, $msg: expr,)*) => {
        $(
            pub const $name: i32 = -$num;
        )*
        #[must_use]
        pub fn as_str(code: i32) -> &'static str {
            match code {
                $(
                    -$num => $msg,
                )*
                _ => "Code not recognized as a Linux error code"
            }
        }
    };
}

#[macro_export]
macro_rules! transparent_bitflags {
    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty {
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

            $(
                $(#[$inner $($args)*])*
                $vis const $Flag: Self = Self($value);
            )*

            #[inline]
            #[allow(dead_code)]
            #[must_use]
            $vis const fn bits(&self) -> $T {
                self.0
            }

            #[inline]
            #[allow(dead_code)]
            #[must_use]
            $vis const fn empty() -> Self {
                Self(0)
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


        impl core::ops::Not for $BitFlags {
            type Output = Self;

            fn not(self) -> Self::Output {
                Self(!self.0)
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
