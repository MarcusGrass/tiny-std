use crate::Error;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use core::hash::Hasher;

use crate::platform::NULL_BYTE;
use crate::string::strlen::strlen;

#[cfg(feature = "alloc")]
#[repr(transparent)]
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UnixString(pub(crate) Vec<u8>);

#[cfg(feature = "alloc")]
impl Debug for UnixString {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let slice = unsafe { core::slice::from_raw_parts(self.0.as_ptr(), self.0.len()) };
        match core::str::from_utf8(slice) {
            Ok(raw) => f.write_fmt(format_args!("UnixString({raw})")),
            Err(_e) => f.write_fmt(format_args!("UnixString({slice:?})")),
        }
    }
}

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
                    unsafe { Ok(core::mem::transmute::<Vec<u8>, Self>(s)) }
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
                    unsafe { Ok(core::mem::transmute::<Vec<u8>, Self>(s.to_vec())) }
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

    /// A slightly less efficient that creating a format string way of
    /// creating a [`UnixString`], since it'll in all likelihood lead to two allocations.
    /// Can't do much since fmt internals are feature gated in `core`.
    /// Still a bit more ergonomic than creating a format-String and then creating a [`UnixString`] from that.
    /// More efficient if a null byte is added to the format strings.
    /// # Example
    /// ```
    /// use rusl::string::unix_str::UnixString;
    /// fn create_format_unix_string() {
    ///     let ins_with = "gramar";
    ///     let good = UnixString::from_format(format_args!("/home/{ins_with}/code"));
    ///     assert_eq!("/home/gramar/code", good.as_str().unwrap());
    ///     let great = UnixString::from_format(format_args!("/home/{ins_with}/code\0"));
    ///     assert_eq!("/home/gramar/code", great.as_str().unwrap());
    /// }
    /// ```
    #[must_use]
    pub fn from_format(args: core::fmt::Arguments<'_>) -> Self {
        let mut fmt_str_buf = alloc::fmt::format(args).into_bytes();
        if !matches!(fmt_str_buf.last(), Some(&NULL_BYTE)) {
            fmt_str_buf.push(NULL_BYTE);
        }
        UnixString(fmt_str_buf)
    }
}

#[cfg(feature = "alloc")]
impl core::ops::Deref for UnixString {
    type Target = UnixStr;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(core::ptr::from_ref::<[u8]>(self.0.as_slice()) as *const UnixStr) }
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
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct UnixStr(pub(crate) [u8]);

impl UnixStr {
    pub const EMPTY: &'static Self = UnixStr::from_str_checked("\0");

    /// # Safety
    /// `&str` needs to be null terminated or downstream UB may occur
    #[inline]
    #[must_use]
    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        core::mem::transmute(s)
    }

    /// # Safety
    /// `&[u8]` needs to be null terminated or downstream UB may occur
    #[inline]
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
                    unsafe { Ok(&*(core::ptr::from_ref::<[u8]>(s) as *const UnixStr)) }
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
        &*(core::ptr::from_ref::<[u8]>(slice) as *const Self)
    }

    /// Try to convert this `&UnixStr` to a utf8 `&str`
    /// # Errors
    /// Not utf8
    pub fn as_str(&self) -> Result<&str, Error> {
        let slice = unsafe { core::slice::from_raw_parts(self.0.as_ptr(), self.0.len() - 1) };
        Ok(core::str::from_utf8(slice)?)
    }

    /// Get this `&UnixStr` as a slice, including the null byte
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
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

    #[must_use]
    pub fn find(&self, other: &Self) -> Option<usize> {
        if other.len() > self.len() {
            return None;
        }
        let this_buf = &self.0;
        let other_buf = &other.0[..other.0.len() - 2];
        buf_find(this_buf, other_buf)
    }

    #[must_use]
    pub fn find_buf(&self, other: &[u8]) -> Option<usize> {
        if other.len() > self.len() {
            return None;
        }
        let this_buf = &self.0;
        buf_find(this_buf, other)
    }

    #[must_use]
    pub fn ends_with(&self, other: &Self) -> bool {
        if other.len() > self.len() {
            return false;
        }
        let mut ind = 0;
        while let (Some(this), Some(that)) = (
            self.0.get(self.0.len() - 1 - ind),
            other.0.get(other.0.len() - 1 - ind),
        ) {
            if this != that {
                return false;
            }
            if other.0.len() - 1 - ind == 0 {
                return true;
            }
            ind += 1;
        }
        true
    }

    /// Get the last component of a path, if possible.
    ///
    /// # Example
    /// ```
    /// use rusl::string::unix_str::UnixStr;
    /// use rusl::unix_lit;
    /// fn get_file_paths() {
    ///     // Has no filename, just a root path
    ///     let a = unix_lit!("/");
    ///     assert_eq!(None, a.path_file_name());
    ///     // Has a 'filename'
    ///     let a = unix_lit!("/etc");
    ///     assert_eq!(Some(unix_lit!("etc")), a.path_file_name());
    /// }
    /// ```
    #[must_use]
    #[allow(clippy::borrow_as_ptr)]
    pub fn path_file_name(&self) -> Option<&UnixStr> {
        for (ind, byte) in self.0.iter().enumerate().rev() {
            if *byte == b'/' {
                return if ind + 2 < self.len() {
                    unsafe {
                        Some(&*(core::ptr::from_ref::<[u8]>(&self.0[ind + 1..]) as *const Self))
                    }
                } else {
                    None
                };
            }
        }
        None
    }

    /// Joins this [`UnixStr`] with some other [`UnixStr`] adding a slash if necessary.
    /// Will make sure that there's at most one slash at the boundary but won't check
    /// either string for "path validity" in any other case
    /// # Example
    /// ```
    /// use rusl::string::unix_str::UnixStr;
    /// fn join_paths() {
    ///     // Combines slash
    ///     let a = UnixStr::try_from_str("path/").unwrap();
    ///     let b = UnixStr::try_from_str("/ext").unwrap();
    ///     let combined = a.path_join(b);
    ///     assert_eq!("path/ext", combined.as_str().unwrap());
    ///     // Adds slash
    ///     let combined_other_way = b.path_join(a);
    ///     assert_eq!("/ext/path/", combined_other_way.as_str().unwrap());
    ///     // Doesn't truncate other slashes, only works at the boundary between the two paths
    ///     let a = UnixStr::try_from_str("path//").unwrap();
    ///     let combined_many_slashes = a.path_join(b);
    ///     assert_eq!("path//ext", combined_many_slashes.as_str().unwrap());
    /// }
    /// ```
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn path_join(&self, ext: &Self) -> UnixString {
        let mut as_string = self.0.to_vec();
        as_string.pop();
        let Some(last) = as_string.last().copied() else {
            return UnixString::from(ext);
        };
        if ext.len() == 1 {
            return UnixString::from(self);
        }
        as_string.reserve(ext.len());
        let buf = if last == b'/' {
            unsafe {
                if ext.0.get_unchecked(0) == &b'/' {
                    as_string.extend_from_slice(ext.0.get_unchecked(1..));
                } else {
                    as_string.extend_from_slice(&ext.0);
                }
            }
            as_string
        } else if unsafe { ext.0.get_unchecked(0) == &b'/' } {
            as_string.extend_from_slice(&ext.0);
            as_string
        } else {
            as_string.push(b'/');
            as_string.extend_from_slice(&ext.0);
            as_string
        };
        UnixString(buf)
    }

    /// Joins this [`UnixStr`] with some format string adding a slash if necessary.
    /// Follows the same rules as [`UnixStr::path_join`].
    /// # Example
    /// ```
    /// use rusl::string::unix_str::UnixStr;
    /// fn join_paths() {
    ///     // Combines slash
    ///     let a = UnixStr::try_from_str("path/").unwrap();
    ///     let combined = a.path_join_fmt(format_args!("ext"));
    ///     assert_eq!("path/ext", combined.as_str().unwrap());
    /// }
    /// ```
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn path_join_fmt(&self, args: core::fmt::Arguments<'_>) -> UnixString {
        let container = alloc::fmt::format(args);
        if container.is_empty() {
            return UnixString::from(self);
        }
        let mut container_vec = container.into_bytes();
        let mut as_string = self.0.to_vec();
        as_string.pop();
        let Some(last) = as_string.last().copied() else {
            if !matches!(container_vec.last().copied(), Some(NULL_BYTE)) {
                container_vec.push(NULL_BYTE);
            }
            return UnixString(container_vec);
        };
        if last == b'/' {
            as_string.reserve(container_vec.len() + 1);
            let start_from = if let Some(b'/') = container_vec.first().copied() {
                1
            } else {
                0
            };
            if let Some(add_slice) = container_vec.get(start_from..) {
                as_string.extend_from_slice(add_slice);
            }
            if !matches!(as_string.last().copied(), Some(NULL_BYTE)) {
                as_string.push(NULL_BYTE);
            }
        } else if let Some(b'/') = container_vec.first().copied() {
            as_string.extend(container_vec);
            if !matches!(as_string.last().copied(), Some(NULL_BYTE)) {
                as_string.push(NULL_BYTE);
            }
        } else {
            as_string.push(b'/');
            as_string.extend(container_vec);
            if !matches!(as_string.last().copied(), Some(NULL_BYTE)) {
                as_string.push(NULL_BYTE);
            }
        }
        UnixString(as_string)
    }

    /// Treats this [`UnixStr`] as a path, then tries to find its parent.
    /// Will treat any double slash as a path with no parent
    /// # Example
    /// ```
    /// use rusl::string::unix_str::UnixStr;
    /// fn find_parent() {
    ///     let well_formed = UnixStr::try_from_str("/home/gramar/code/").unwrap();
    ///     let up_one = well_formed.parent_path().unwrap();
    ///     assert_eq!("/home/gramar", up_one.as_str().unwrap());
    ///     let up_two = up_one.parent_path().unwrap();
    ///     assert_eq!("/home", up_two.as_str().unwrap());
    ///     let up_three = up_two.parent_path().unwrap();
    ///     assert_eq!("/", up_three.as_str().unwrap());
    ///     assert!(up_three.parent_path().is_none());
    ///     let ill_formed = UnixStr::try_from_str("/home/gramar/code//").unwrap();
    ///     assert!(ill_formed.parent_path().is_none());
    /// }
    /// ```
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn parent_path(&self) -> Option<UnixString> {
        let len = self.0.len();
        // Can't be len 0, len 1 is only a null byte, len 2 is a single char, parent becomes none
        if len < 3 {
            return None;
        }
        let last = self.0.len() - 2;
        let mut next_slash_back = last;
        while let Some(byte) = self.0.get(next_slash_back).copied() {
            if byte == b'/' {
                if next_slash_back != 0 {
                    if let Some(b'/') = self.0.get(next_slash_back - 1) {
                        return None;
                    }
                }
                break;
            }
            if next_slash_back == 0 {
                return None;
            }
            next_slash_back -= 1;
        }
        // Found slash at root, we want to include it in output then giving only the root path
        if next_slash_back == 0 {
            next_slash_back += 1;
        }
        unsafe {
            Some(UnixString(
                self.0.get_unchecked(..=next_slash_back).to_vec(),
            ))
        }
    }
}

#[inline]
#[allow(clippy::needless_range_loop)]
fn buf_find(this_buf: &[u8], other_buf: &[u8]) -> Option<usize> {
    for i in 0..this_buf.len() {
        if this_buf[i] == other_buf[0] {
            let mut no_match = false;
            for j in 1..other_buf.len() {
                if let Some(this) = this_buf.get(i + j) {
                    if *this != other_buf[j] {
                        no_match = true;
                        break;
                    }
                } else {
                    return None;
                }
            }
            if !no_match {
                return Some(i);
            }
        }
    }
    None
}

impl<'a> Debug for &'a UnixStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let slice = unsafe { core::slice::from_raw_parts(self.0.as_ptr(), self.0.len()) };
        match core::str::from_utf8(slice) {
            Ok(inner) => f.write_fmt(format_args!("UnixStr({inner})")),
            Err(_e) => f.write_fmt(format_args!("UnixStr({slice:?})")),
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

#[cfg(feature = "alloc")]
impl core::str::FromStr for UnixString {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s)
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

    #[test]
    fn can_check_ends_with() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        assert!(UNIX_STR.ends_with(UnixStr::from_str_checked("-str\0")));
    }

    #[test]
    fn can_check_ends_with_self() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        assert!(UNIX_STR.ends_with(UNIX_STR));
    }

    #[test]
    fn can_check_ends_with_empty() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        assert!(UNIX_STR.ends_with(UnixStr::EMPTY));
    }

    #[test]
    fn can_check_ends_with_no() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        assert!(!UNIX_STR.ends_with(UnixStr::from_str_checked("nice-\0")));
    }

    #[test]
    fn can_check_ends_with_no_too_long() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        assert!(!UNIX_STR.ends_with(UnixStr::from_str_checked("other-my-nice-str\0")));
    }

    #[test]
    fn can_find_at_end() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        let found_at = UNIX_STR.find(UnixStr::from_str_checked("-str\0")).unwrap();
        assert_eq!(7, found_at);
    }

    #[test]
    fn can_find_finds_first() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("str-str-str\0");
        let found_at = UNIX_STR.find(UnixStr::from_str_checked("-str\0")).unwrap();
        assert_eq!(3, found_at);
        let found_at = UNIX_STR.find_buf("-str".as_bytes()).unwrap();
        assert_eq!(3, found_at);
    }

    #[test]
    fn can_find_at_middle() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        let found_at = UNIX_STR.find(UnixStr::from_str_checked("-nice\0")).unwrap();
        assert_eq!(2, found_at);
        let found_at = UNIX_STR.find_buf("-nice".as_bytes()).unwrap();
        assert_eq!(2, found_at);
    }

    #[test]
    fn can_find_at_start() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        let found_at = UNIX_STR.find(UnixStr::from_str_checked("my\0")).unwrap();
        assert_eq!(0, found_at);
        let found_at = UNIX_STR.find_buf("my".as_bytes()).unwrap();
        assert_eq!(0, found_at);
    }

    #[test]
    fn can_find_no_match() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("my-nice-str\0");
        let found_at = UNIX_STR.find(UnixStr::from_str_checked("cake\0"));
        assert!(found_at.is_none());
        let found_at = UNIX_STR.find_buf("cake".as_bytes());
        assert!(found_at.is_none());
    }

    #[test]
    fn can_find_too_long() {
        const UNIX_STR: &UnixStr = UnixStr::from_str_checked("str\0");
        let found_at = UNIX_STR.find(UnixStr::from_str_checked("sstr\0"));
        assert!(found_at.is_none());

        let found_at = UNIX_STR.find_buf("sstr".as_bytes());
        assert!(found_at.is_none());
    }

    #[cfg(feature = "alloc")]
    mod alloc_tests {
        use super::*;
        use alloc::string::ToString;
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
        fn can_path_join() {
            let a = UnixStr::from_str_checked("hello\0");
            let b = UnixStr::from_str_checked("there\0");
            let new = a.path_join(b);
            assert_eq!("hello/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_fmt() {
            let a = UnixStr::from_str_checked("hello\0");
            let new = a.path_join_fmt(format_args!("there"));
            assert_eq!("hello/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_with_trailing_slash() {
            let a = UnixStr::from_str_checked("hello/\0");
            let b = UnixStr::from_str_checked("there\0");
            let new = a.path_join(b);
            assert_eq!("hello/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_fmt_with_trailing_slash() {
            let a = UnixStr::from_str_checked("hello/\0");
            let new = a.path_join_fmt(format_args!("there"));
            assert_eq!("hello/there", new.as_str().unwrap());
        }
        #[test]
        fn can_path_join_with_leading_slash() {
            let a = UnixStr::from_str_checked("hello\0");
            let b = UnixStr::from_str_checked("/there\0");
            let new = a.path_join(b);
            assert_eq!("hello/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_fmt_with_leading_slash() {
            let a = UnixStr::from_str_checked("hello\0");
            let new = a.path_join_fmt(format_args!("/there"));
            assert_eq!("hello/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_empty() {
            let a = UnixStr::from_str_checked("\0");
            let b = UnixStr::from_str_checked("/there\0");
            let new = a.path_join(b);
            assert_eq!("/there", new.as_str().unwrap());
            let new = b.path_join(a);
            assert_eq!("/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_fmt_empty() {
            let a = UnixStr::from_str_checked("\0");
            let b = UnixStr::from_str_checked("/there\0");
            let new = a.path_join_fmt(format_args!("/there"));
            assert_eq!("/there", new.as_str().unwrap());
            let new = b.path_join_fmt(format_args!(""));
            assert_eq!("/there", new.as_str().unwrap());
        }

        #[test]
        fn can_path_join_truncates_slashes() {
            let a = UnixStr::from_str_checked("hello/\0");
            let b = UnixStr::from_str_checked("/there\0");
            let new = a.path_join(b);
            assert_eq!("hello/there", new.as_str().unwrap());
        }

        #[test]
        fn can_get_last_path_happy() {
            let base = unix_lit!("a/b/c");
            let res = base.path_file_name().unwrap();
            let expect = unix_lit!("c");
            assert_eq!(expect, res);
        }

        #[test]
        fn can_get_last_path_root_gives_none() {
            let base = unix_lit!("/");
            assert!(base.path_file_name().is_none());
        }

        #[test]
        fn can_get_last_path_empty_gives_none() {
            assert!(UnixStr::EMPTY.path_file_name().is_none());
        }

        #[test]
        fn find_parent_path_happy() {
            let a = UnixStr::from_str_checked("hello/there/friend\0");
            let parent = a.parent_path().unwrap();
            assert_eq!("hello/there", parent.as_str().unwrap());
            let b = UnixStr::from_str_checked("/home/gramar/code/rust/tiny-std\0");
            let b_first_parent = b.parent_path().unwrap();
            assert_eq!("/home/gramar/code/rust", b_first_parent.as_str().unwrap());
            let b_second_parent = b_first_parent.parent_path().unwrap();
            assert_eq!("/home/gramar/code", b_second_parent.as_str().unwrap());
            let b_third_parent = b_second_parent.parent_path().unwrap();
            assert_eq!("/home/gramar", b_third_parent.as_str().unwrap());
            let b_fourth_parent = b_third_parent.parent_path().unwrap();
            assert_eq!("/home", b_fourth_parent.as_str().unwrap());
            let root = b_fourth_parent.parent_path().unwrap();
            assert_eq!("/", root.as_str().unwrap());
        }

        #[test]
        fn find_parent_path_empty_no_parent() {
            let a = UnixStr::EMPTY;
            let parent = a.parent_path();
            assert!(parent.is_none());
        }

        #[test]
        fn find_parent_path_short_no_parent() {
            let a = UnixStr::from_str_checked("/\0");
            let parent = a.parent_path();
            assert!(parent.is_none());
            let b = UnixStr::from_str_checked("a\0");
            let parent = b.parent_path();
            assert!(parent.is_none());
        }

        #[test]
        fn find_parent_path_short_has_parent() {
            let a = UnixStr::from_str_checked("/a\0");
            let parent = a.parent_path().unwrap();
            assert_eq!("/", parent.as_str().unwrap());
        }

        #[test]
        fn find_parent_path_double_slash_invalid() {
            let a = UnixStr::from_str_checked("//\0");
            let parent = a.parent_path();
            assert!(parent.is_none());
            let a = UnixStr::from_str_checked("hello//\0");
            let parent = a.parent_path();
            assert!(parent.is_none());
        }

        #[test]
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
    }
}
