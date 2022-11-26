#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use core::str::Utf8Error;

use crate::platform::{NULL_BYTE, NULL_CHAR};
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

pub trait AsMutUnixStr: AsUnixStr {
    /// Executes a function with this null terminated entity
    /// converts it to a string and pushes a null byte if not already null terminated
    /// # Errors
    /// Propagates the function's errors
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>;
}

#[cfg(feature = "alloc")]
pub trait ToUnixString {
    fn to_unix_string(&self) -> UnixString;
}

#[cfg(feature = "alloc")]
impl ToUnixString for () {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        UnixString(vec![b'\0'])
    }
}

impl AsUnixStr for () {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        func("\0".as_ptr())
    }

    #[inline]
    unsafe fn match_up_to(&self, _null_terminated_pointer: *const u8) -> usize {
        0
    }
}

impl AsMutUnixStr for () {
    #[inline]
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        func([NULL_BYTE].as_mut_ptr())
    }
}

#[cfg(feature = "alloc")]
impl<T> ToUnixString for &T
where
    T: ToUnixString,
{
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        self.deref().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl<T> ToUnixString for &mut T
where
    T: ToUnixString,
{
    #[inline]
    fn to_unix_string(&self) -> UnixString {
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

impl<A> AsMutUnixStr for &mut A
where
    A: AsMutUnixStr,
{
    #[inline]
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        self.deref_mut().exec_with_self_as_mut_ptr(func)
    }
}

#[cfg(feature = "alloc")]
#[repr(transparent)]
#[derive(Debug, Clone)]
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
impl From<String> for UnixString {
    #[inline]
    fn from(mut s: String) -> Self {
        if !s.ends_with(NULL_CHAR) {
            s.push(NULL_CHAR);
        }
        Self(s.into_bytes())
    }
}

#[cfg(feature = "alloc")]
impl From<&str> for UnixString {
    #[inline]
    fn from(s: &str) -> Self {
        let owned: String = s.to_string();
        owned.into()
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &str {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        let owned = (*self).to_string();
        owned.to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl From<Vec<u8>> for UnixString {
    #[inline]
    fn from(mut buf: Vec<u8>) -> Self {
        let len = buf.len();
        if len == 0 {
            buf.push(NULL_BYTE);
            UnixString(buf)
        } else if unsafe { *buf.get_unchecked(len - 1) == NULL_BYTE } {
            unsafe { core::mem::transmute(buf) }
        } else {
            buf.push(NULL_BYTE);
            UnixString(buf)
        }
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for Vec<u8> {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        self.as_slice().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl AsUnixStr for Vec<u8> {
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        let len = self.len();
        if len == 0 {
            func([NULL_BYTE].as_ptr())
        } else if unsafe { *self.get_unchecked(len - 1) == NULL_BYTE } {
            func(self.as_ptr())
        } else {
            let mut buf = self.clone();
            buf.push(NULL_BYTE);
            func(self.as_ptr())
        }
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.as_slice().match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl AsMutUnixStr for Vec<u8> {
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        let len = self.len();
        if len == 0 {
            func([NULL_BYTE].as_mut_ptr())
        } else if unsafe { *self.get_unchecked(len - 1) == NULL_BYTE } {
            func(self.as_mut_ptr())
        } else {
            let mut buf = self.clone();
            buf.push(NULL_BYTE);
            func(self.as_mut_ptr())
        }
    }
}

impl AsUnixStr for &[u8] {
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        let len = self.len();
        if len == 0 {
            func([NULL_BYTE].as_ptr())
        } else if unsafe { *self.get_unchecked(len - 1) == NULL_BYTE } {
            func(self.as_ptr())
        } else {
            #[cfg(feature = "alloc")]
            {
                let mut buf = self.to_vec();
                buf.push(NULL_BYTE);
                func(self.as_ptr())
            }
            #[cfg(not(feature = "alloc"))]
            Err(crate::Error::not_null_terminated())
        }
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        let this_len = self.len();
        for i in 0..this_len {
            let other_byte = null_terminated_pointer.add(i).read();
            if self[i] != other_byte {
                return i;
            }
        }
        this_len - 1
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &mut [u8] {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
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

impl AsMutUnixStr for &mut [u8] {
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        let len = self.len();
        if len == 0 {
            func([NULL_BYTE].as_mut_ptr())
        } else if unsafe { *self.get_unchecked(len - 1) == NULL_BYTE } {
            func(self.as_mut_ptr())
        } else {
            #[cfg(feature = "alloc")]
            {
                let mut buf = self.to_vec();
                buf.push(NULL_BYTE);
                func(self.as_mut_ptr())
            }
            #[cfg(not(feature = "alloc"))]
            Err(crate::Error::not_null_terminated())
        }
    }
}

#[cfg(feature = "alloc")]
impl From<&[u8]> for UnixString {
    #[inline]
    fn from(buf: &[u8]) -> Self {
        buf.to_vec().into()
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &[u8] {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        let mut v = self.to_vec();
        if v.ends_with(&[NULL_BYTE]) {
            UnixString(v)
        } else {
            v.push(NULL_BYTE);
            UnixString(v)
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
    fn from(s: &UnixStr) -> Self {
        s.0.to_vec().into()
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &UnixStr {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        UnixString(self.0.to_vec())
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

impl AsMutUnixStr for &mut UnixStr {
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        func(self.0.as_mut_ptr())
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for &mut UnixStr {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
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
impl AsMutUnixStr for UnixString {
    #[inline]
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        func(self.0.as_mut_ptr())
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for UnixString {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        self.clone()
    }
}

impl AsUnixStr for &str {
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        if self.ends_with(NULL_CHAR) {
            return func(self.as_ptr());
        }
        #[cfg(feature = "alloc")]
        {
            let mut owned = (*self).to_string();
            owned.push(NULL_CHAR);
            func(owned.as_ptr())
        }

        #[cfg(not(feature = "alloc"))]
        {
            if self.is_empty() {
                return func(b"\0".as_ptr());
            }
            Err(crate::Error::no_code("str not null terminated"))
        }
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
    fn to_unix_string(&self) -> UnixString {
        self.deref().to_unix_string()
    }
}

impl AsMutUnixStr for &mut str {
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        if self.ends_with(NULL_CHAR) {
            return func(self.as_mut_ptr());
        }
        #[cfg(feature = "alloc")]
        {
            let mut owned = (*(*self)).to_string();
            owned.push(NULL_CHAR);
            func(owned.as_mut_ptr())
        }

        #[cfg(not(feature = "alloc"))]
        {
            if self.is_empty() {
                return func([NULL_BYTE].as_mut_ptr());
            }
            Err(crate::Error::no_code("str not null terminated"))
        }
    }
}

#[cfg(feature = "alloc")]
impl AsUnixStr for String {
    #[inline]
    fn exec_with_self_as_ptr<F, T>(&self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*const u8) -> crate::Result<T>,
    {
        if self.ends_with(NULL_CHAR) {
            func(self.as_ptr())
        } else {
            let mut clone = self.clone();
            clone.push(NULL_CHAR);
            func(clone.as_ptr())
        }
    }

    #[inline]
    unsafe fn match_up_to(&self, null_terminated_pointer: *const u8) -> usize {
        self.as_bytes().match_up_to(null_terminated_pointer)
    }
}

#[cfg(feature = "alloc")]
impl ToUnixString for String {
    #[inline]
    fn to_unix_string(&self) -> UnixString {
        self.as_bytes().to_unix_string()
    }
}

#[cfg(feature = "alloc")]
impl AsMutUnixStr for String {
    fn exec_with_self_as_mut_ptr<F, T>(&mut self, func: F) -> crate::Result<T>
    where
        F: FnOnce(*mut u8) -> crate::Result<T>,
    {
        if self.ends_with(NULL_CHAR) {
            func(self.as_mut_ptr())
        } else {
            let mut clone = self.clone();
            clone.push(NULL_CHAR);
            func(self.as_mut_ptr())
        }
    }
}
