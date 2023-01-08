use crate::Error;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::ops::Deref;
use core::str::Utf8Error;

use crate::platform::NULL_BYTE;
use crate::string::null_term_ptr_cmp_up_to;
use crate::string::strlen::strlen;

#[cfg(feature = "alloc")]
pub trait AsUnixStr: ToUnixString {
    /// Executes a function with this null terminated entity
    /// converts it to a string and pushes a null byte if not already null terminated
    /// # Errors
    /// Propagates the function's errors
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>;

    /// Checks if this `AsUnixStr` matches a null terminated pointer and returns the non null length
    /// # Safety
    /// Pointer is null terminated
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize;
}

#[cfg(not(feature = "alloc"))]
pub trait AsUnixStr {
    /// Executes a function with this null terminated entity
    /// # Errors
    /// 1. Propagates the function's errors
    /// 2. Errors if the provided entity isn't null terminated as we need an allocator to modify
    /// it.
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>;

    /// Checks if this `AsUnixStr` matches a null terminated pointer and returns the non null length
    /// # Safety
    /// Pointer is null terminated
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize;
}

#[cfg(feature = "alloc")]
pub trait ToUnixString {
    /// Turn this into a `UnixString`
    /// # Errors
    /// If this string can't be converted this will throw an error
    /// The only real reasons are if you have multiple null bytes or no access to an allocator
    fn to_unix_string(&self) -> crate::Result<UnixString>;
}

#[cfg(feature = "alloc")]
impl ToUnixString for () {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        Ok(UnixString(vec![b'\0']))
    }
}

impl AsUnixStr for () {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        func([NULL_BYTE].as_ptr())
    }

    #[inline]
    unsafe fn match_up_to(&self, _null_terminated_pointer: *const u8) -> usize {
        0
    }
}

#[cfg(feature = "alloc")]
impl<T> ToUnixString for &T
where
    T: ToUnixString,
{
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.deref().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl<T> ToUnixString for &mut T
where
    T: ToUnixString,
{
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.deref().to_unix_string()
    }
}

impl<A> AsUnixStr for &A
where
    A: AsUnixStr,
{
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.deref().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.deref().match_up_to(null_terminated_pointer)
    }
}

impl<A> AsUnixStr for &mut A
where
    A: AsUnixStr,
{
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.deref().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.deref().match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnixString(pub(crate) Vec<u8>);

#[cfg(feature = "alloc")]
impl UnixString {
    #[inline]
    #[must_use]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}

#[cfg(feature = "alloc")]
impl TryFrom<String> for UnixString {
    type Error = crate::Error;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.into_bytes().try_into()
    }
}

#[cfg(feature = "alloc")]
impl TryFrom<&str> for UnixString {
    type Error = crate::Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().to_vec().try_into()
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &str {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.as_bytes().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl TryFrom<Vec<u8>> for UnixString {
    type Error = crate::Error;
    fn try_from(mut value: Vec<u8>) -> Result<Self, Self::Error> {
        match null_terminated_ok(&value) {
            NullTermCheckResult::NullTerminated => Ok(Self(value)),
            NullTermCheckResult::NullByteOutOfPlace => null_byte_out_of_place(),
            NullTermCheckResult::NotNullTerminated => {
                value.push(NULL_BYTE);
                Ok(Self(value))
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for Vec<u8> {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.as_slice().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl AsUnixStr for Vec<u8> {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.as_slice().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.as_slice().match_up_to(null_terminated_pointer)
    }
}

impl AsUnixStr for &[u8] {
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        match null_terminated_ok(self) {
            NullTermCheckResult::NullTerminated => func(self.as_ptr()),
            NullTermCheckResult::NullByteOutOfPlace => null_byte_out_of_place(),
            NullTermCheckResult::NotNullTerminated => {
                if self.is_empty() {
                    return func([NULL_BYTE].as_ptr());
                }
                #[cfg(feature = "alloc")]
                {
                    let mut buf = self.to_vec();
                    buf.push(NULL_BYTE);
                    func(buf.as_ptr())
                }
                #[cfg(not(feature = "alloc"))]
                Err(crate::Error::not_null_terminated())
            }
        }
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        let this_len = self.len();
        for i in 0..this_len {
            let other_byte = null_terminated_pointer.add(i).read();
            if self[i] != other_byte || other_byte == NULL_BYTE {
                return i;
            }
        }
        this_len - 1
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &mut [u8] {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.deref().to_unix_string()
    }
}

impl AsUnixStr for &mut [u8] {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.deref().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        (&self).match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl TryFrom<&[u8]> for UnixString {
    type Error = crate::Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value.to_vec().try_into()
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &[u8] {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        match null_terminated_ok(self) {
            NullTermCheckResult::NullTerminated => Ok(UnixString(self.to_vec())),
            NullTermCheckResult::NullByteOutOfPlace => null_byte_out_of_place(),
            NullTermCheckResult::NotNullTerminated => {
                let mut v = self.to_vec();
                v.push(NULL_BYTE);
                Ok(UnixString(v))
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl core::ops::Deref for UnixString {
    type Target = UnixStr;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0.as_slice() as *const [u8] as *const UnixStr) }
    }
}

#[repr(transparent)]
#[derive(Eq, PartialEq)]
pub struct UnixStr(pub(crate) [u8]);

impl UnixStr {
    /// # Safety
    /// `&str` needs to be null terminated or downstream UB may occur
    #[must_use]
    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        core::mem::transmute(s)
    }

    /// # Safety
    /// `s` needs to be null terminated
    #[must_use]
    pub unsafe fn from_ptr<'a>(s: *const u8) -> &'a Self {
        let non_null_len = strlen(s);
        let slice = core::slice::from_raw_parts(s, non_null_len + 1);
        core::mem::transmute(slice)
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
}

#[cfg(feature = "alloc")]
impl From<&UnixStr> for UnixString {
    #[inline]
    fn from(s: &UnixStr) -> Self {
        UnixString(s.0.to_vec())
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &UnixStr {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        Ok(UnixString(self.0.to_vec()))
    }
}

impl AsUnixStr for &UnixStr {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        func(self.0.as_ptr())
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        null_term_ptr_cmp_up_to(self.0.as_ptr(), null_terminated_pointer)
    }
}

impl AsUnixStr for &mut UnixStr {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.deref().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        null_term_ptr_cmp_up_to(self.0.as_ptr(), null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &mut UnixStr {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.deref().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl AsUnixStr for UnixString {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        func(self.0.as_ptr())
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.deref().match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for UnixString {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        Ok(self.clone())
    }
}

impl AsUnixStr for &str {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.as_bytes().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.as_bytes().match_up_to(null_terminated_pointer)
    }
}

impl AsUnixStr for &mut str {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.deref().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.as_bytes().match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &mut str {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.deref().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl AsUnixStr for String {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        self.as_bytes().exec_with_self_as_ptr(func)
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.as_bytes().match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for String {
    #[inline]
    fn to_unix_string(&self) -> crate::Result<UnixString> {
        self.as_bytes().to_unix_string()
    }
}

const fn null_byte_out_of_place<T>() -> crate::Result<T> {
    Err(Error::no_code("Null byte out of place"))
}

#[derive(Debug, Copy, Clone)]
enum NullTermCheckResult {
    NullTerminated,
    NullByteOutOfPlace,
    NotNullTerminated,
}

#[inline]
fn null_terminated_ok(s: &[u8]) -> NullTermCheckResult {
    let len = s.len();
    for (ind, byte) in s.iter().enumerate() {
        if *byte == NULL_BYTE {
            return if ind == len - 1 {
                NullTermCheckResult::NullTerminated
            } else {
                NullTermCheckResult::NullByteOutOfPlace
            };
        }
    }
    NullTermCheckResult::NotNullTerminated
}
