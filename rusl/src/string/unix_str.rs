use crate::Error;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::hash::Hasher;
use core::str::Utf8Error;

use crate::platform::NULL_BYTE;
use crate::string::strlen::strlen;

#[cfg(feature = "alloc")]
#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UnixString(pub(crate) Vec<u8>);

#[cfg(feature = "alloc")]
impl UnixString {
    #[inline]
    #[must_use]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Create a `UnixString` from a `String`.
    /// # Errors
    /// String has nulls in other places than end.
    #[inline]
    pub fn try_from_string(s: String) -> Result<Self, Error> {
        Self::try_from_vec(s.into_bytes())
    }

    /// Create a `UnixString` from a `Vec<u8>`.
    /// # Errors
    /// Vec has nulls in other places than end.
    #[inline]
    pub fn try_from_vec(mut s: Vec<u8>) -> Result<Self, Error> {
        let len = s.len();
        for (ind, byte) in s.iter().enumerate() {
            if *byte == NULL_BYTE {
                return if ind == len - 1 {
                    unsafe { Ok(core::mem::transmute(s)) }
                } else {
                    Err(Error::no_code("Tried to instantiate UnixStr from an invalid &str, a null byte was found but out of place"))
                };
            }
        }
        s.push(0);
        Ok(Self(s))
    }

    /// Create a `UnixString` from a `&[u8]`.
    /// Will allocate and push a null byte if not null terminated
    /// # Errors
    /// Bytes aren't properly null terminated, several nulls contained.
    pub fn try_from_bytes(s: &[u8]) -> Result<Self, Error> {
        let len = s.len();
        for (ind, byte) in s.iter().enumerate() {
            if *byte == NULL_BYTE {
                return if ind == len - 1 {
                    unsafe { Ok(core::mem::transmute(s.to_vec())) }
                } else {
                    Err(Error::no_code("Tried to instantiate UnixStr from an invalid &str, a null byte was found but out of place"))
                };
            }
        }
        let mut new = s.to_vec();
        new.push(0);
        Ok(Self(new))
    }

    /// Create a `UnixString` from a `&str`.
    /// Will allocate and push a null byte if not null terminated
    /// # Errors
    /// String isn't properly null terminated, several nulls contained.
    #[inline]
    pub fn try_from_str(s: &str) -> Result<Self, Error> {
        Self::try_from_bytes(s.as_bytes())
    }
}

#[cfg(feature = "alloc")]
impl core::ops::Deref for UnixString {
    type Target = UnixStr;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0.as_slice() as *const [u8] as *const UnixStr) }
    }
}

#[cfg(feature = "alloc")]
impl AsRef<UnixStr> for UnixString {
    #[inline]
    fn as_ref(&self) -> &UnixStr {
        unsafe { UnixStr::from_bytes_unchecked(self.0.as_slice()) }
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UnixStr(pub(crate) [u8]);

impl UnixStr {
    pub const EMPTY: &'static Self = UnixStr::from_str_checked("\0");

    /// # Safety
    /// `&str` needs to be null terminated or downstream UB may occur
    #[must_use]
    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        core::mem::transmute(s)
    }

    /// # Safety
    /// `&[u8]` needs to be null terminated or downstream UB may occur
    #[must_use]
    pub const unsafe fn from_bytes_unchecked(s: &[u8]) -> &Self {
        core::mem::transmute(s)
    }

    /// Const instantiation of a `&UnixStr` from a `&str`.
    /// Should only be used in a `const`-context, although `rustc` does not let me enforce this.
    /// # Panics
    /// This method panics since it's supposed to produce a comptime error, it's
    /// not particularly efficient.
    #[must_use]
    pub const fn from_str_checked(s: &str) -> &Self {
        const_null_term_validate(s.as_bytes());
        unsafe { core::mem::transmute(s) }
    }

    /// Create a `&UnixStr` from a `&str`.
    /// # Errors
    /// String isn't properly null terminated.
    #[inline]
    pub fn try_from_str(s: &str) -> Result<&Self, Error> {
        Self::try_from_bytes(s.as_bytes())
    }

    /// Create a `&UnixStr` from a `&[u8]`.
    /// # Errors
    /// Slice isn't properly null terminated.
    #[inline]
    pub fn try_from_bytes(s: &[u8]) -> Result<&Self, Error> {
        let len = s.len();
        for (ind, byte) in s.iter().enumerate() {
            if *byte == NULL_BYTE {
                return if ind == len - 1 {
                    unsafe { Ok(&*(s as *const [u8] as *const UnixStr)) }
                } else {
                    Err(Error::no_code("Tried to instantiate UnixStr from an invalid &str, a null byte was found but out of place"))
                };
            }
        }
        Err(Error::no_code(
            "Tried to instantiate UnixStr from an invalid &str, not null terminated",
        ))
    }

    /// # Safety
    /// `s` needs to be null terminated
    #[must_use]
    pub unsafe fn from_ptr<'a>(s: *const u8) -> &'a Self {
        let non_null_len = strlen(s);
        let slice = core::slice::from_raw_parts(s, non_null_len + 1);
        &*(slice as *const [u8] as *const Self)
    }

    /// Try to convert this `&UnixStr` to a utf8 `&str`
    /// # Errors
    /// Not utf8
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        let slice = unsafe { core::slice::from_raw_parts(self.0.as_ptr(), self.0.len() - 1) };
        core::str::from_utf8(slice)
    }

    /// Get this `&UnixStr` as a slice, including the null byte
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Get the length of this `&UnixStr`, including the null byte
    #[inline]
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the length of this `&UnixStr`, including the null byte
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    #[must_use]
    pub fn match_up_to(&self, other: &UnixStr) -> usize {
        let mut it = 0;
        let slf_ptr = self.as_ptr();
        let other_ptr = other.as_ptr();
        loop {
            unsafe {
                let a_val = slf_ptr.add(it).read();
                let b_val = other_ptr.add(it).read();
                if a_val != b_val || a_val == NULL_BYTE {
                    // Not equal, or terminated
                    return it;
                }
                // Equal continue
                it += 1;
            }
        }
    }

    #[must_use]
    pub fn match_up_to_str(&self, other: &str) -> usize {
        let mut it = 0;
        let slf_ptr = self.as_ptr();
        let other_ptr = other.as_ptr();
        let other_len = other.len();
        loop {
            unsafe {
                let a_val = slf_ptr.add(it).read();
                let b_val = other_ptr.add(it).read();
                if a_val != b_val || a_val == NULL_BYTE {
                    // Not equal, or terminated
                    return it;
                }
                // Equal continue
                it += 1;
            }
            if it == other_len {
                return it;
            }
        }
    }
}

impl<'a> core::hash::Hash for &'a UnixStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[cfg(feature = "alloc")]
impl From<&UnixStr> for UnixString {
    #[inline]
    fn from(s: &UnixStr) -> Self {
        UnixString(s.0.to_vec())
    }
}

#[inline]
const fn const_null_term_validate(s: &[u8]) {
    assert!(
        !s.is_empty(),
        "Tried to instantiate UnixStr from an invalid &str, not null terminated"
    );
    let len = s.len() - 1;
    let mut i = len;
    assert!(
        s[i] == b'\0',
        "Tried to instantiate UnixStr from an invalid &str, not null terminated"
    );
    while i > 0 {
        i -= 1;
        assert!(s[i] != b'\0', "Tried to instantiate UnixStr from an invalid &str, a null byte was found but out of place");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_match_up_to() {
        let haystack = UnixStr::try_from_str("haystack\0").unwrap();
        let needle = UnixStr::EMPTY;
        assert_eq!(0, haystack.match_up_to(needle));
        let needle = UnixStr::try_from_str("h\0").unwrap();
        assert_eq!(1, haystack.match_up_to(needle));
        let needle = UnixStr::try_from_str("haystac\0").unwrap();
        assert_eq!(7, haystack.match_up_to(needle));
        let needle = UnixStr::try_from_str("haystack\0").unwrap();
        assert_eq!(8, haystack.match_up_to(needle));
        let needle = UnixStr::try_from_str("haystack2\0").unwrap();
        assert_eq!(8, haystack.match_up_to(needle));
    }
    #[test]
    #[cfg(feature = "alloc")]
    fn can_create_unix_string_happy() {
        let correct = UnixString::try_from_str("abc\0").unwrap();
        let correct2 = UnixString::try_from_bytes(b"abc\0").unwrap();
        let correct3 = UnixString::try_from_string("abc\0".to_string()).unwrap();
        let correct4 = UnixString::try_from_vec(b"abc\0".to_vec()).unwrap();
        let correct5 = UnixString::try_from_str("abc").unwrap();
        let correct6 = UnixString::try_from_string(String::from("abc")).unwrap();
        assert_eq!(correct, correct2);
        assert_eq!(correct2, correct3);
        assert_eq!(correct3, correct4);
        assert_eq!(correct4, correct5);
        assert_eq!(correct5, correct6);
        let compare = [b'a', b'b', b'c', 0];
        assert_eq!(correct4.as_slice(), compare);
        assert_ne!(correct.as_ptr(), correct2.as_ptr());
        assert_eq!(UnixStr::try_from_str("abc\0").unwrap(), correct.as_ref());
        assert_eq!(UnixStr::try_from_str("abc\0").unwrap(), &*correct);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn can_create_unix_string_sad() {
        let acceptable = UnixString::try_from_str("abc").unwrap();
        let correct = UnixString::try_from_str("abc\0").unwrap();
        assert_eq!(correct, acceptable);
        let unacceptable = UnixString::try_from_str("a\0bc");
        assert!(unacceptable.is_err());
        let unacceptable_vec = UnixString::try_from_vec(alloc::vec![b'a', b'\0', b'b', b'c']);
        assert!(unacceptable_vec.is_err());
    }

    #[test]
    fn can_create_unix_str() {
        const CONST_CORRECT: &UnixStr = UnixStr::from_str_checked("abc\0");

        let correct1 = UnixStr::try_from_str("abc\0").unwrap();
        let correct2 = UnixStr::try_from_bytes(b"abc\0").unwrap();
        assert_eq!(CONST_CORRECT, correct1);
        assert_eq!(correct1, correct2);
    }

    #[test]
    fn can_create_unix_str_sad() {
        let unacceptable = UnixStr::try_from_str("a\0bc");
        assert!(unacceptable.is_err());
        let unacceptable_vec = UnixStr::try_from_bytes(&[b'a', b'\0', b'b', b'c']);
        assert!(unacceptable_vec.is_err());
        let unacceptable_not_null_term = UnixStr::try_from_str("abc");
        assert!(unacceptable_not_null_term.is_err());
    }

    #[test]
    fn can_cmp_unix_str_and_str() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        const MY_CMP_STR: &str = "my-nice-str";
        assert_eq!(MY_CMP_STR.len(), UNIX_STR.match_up_to_str(MY_CMP_STR));
    }
}
