use crate::platform::NULL_BYTE;
use crate::string::strlen::{buf_strlen, strlen};
use crate::string::unix_str::AsUnixStr;

#[test]
#[cfg(not(feature = "alloc"))]
fn errs_on_not_null_without_alloc() {
    let use_with_non_null_without_allocator = "Hello".exec_with_self_as_ptr(|_| Ok(()));
    assert!(use_with_non_null_without_allocator.is_err());
}

#[test]
#[cfg(feature = "alloc")]
fn accepts_not_null_on_alloc() {
    let use_with_non_null_without_allocator = "Hello".exec_with_self_as_ptr(|_| Ok(()));
    assert!(use_with_non_null_without_allocator.is_ok());
}

#[test]
fn errs_on_many_null() {
    let raw = "Hello\0oh no\0";
    let many_non_null_res = raw.exec_with_self_as_ptr(|_| Ok(()));
    assert!(many_non_null_res.is_err());
}

#[test]
fn accepts_empty() {
    let empty = "".exec_with_self_as_ptr(|ptr| {
        let null_byte = unsafe { ptr.read() };
        assert_eq!(NULL_BYTE, null_byte);
        Ok(())
    });
    assert!(empty.is_ok());
}

#[test]
#[cfg(feature = "alloc")]
fn accepts_non_null_terminated_with_allocator() {
    use alloc::borrow::ToOwned;
    let owned = "Hello".to_owned();
    let non_null_term_with_alloc = owned.exec_with_self_as_ptr(|_| Ok(()));
    assert!(non_null_term_with_alloc.is_ok());
}

#[test]
#[cfg(feature = "alloc")]
fn can_convert_into_unix_string() {
    use crate::string::unix_str::UnixString;
    let template = UnixString::try_from("Hello!\0").unwrap();
    let owned_with_null = b"Hello!\0".to_vec();
    assert_eq!(
        template,
        UnixString::try_from(owned_with_null.clone()).unwrap()
    );
    let owned_non_null = b"Hello!".to_vec();
    assert_eq!(
        template,
        UnixString::try_from(owned_non_null.clone()).unwrap()
    );
    let owned_empty = b"".to_vec();
    let template_empty = UnixString::try_from("\0").unwrap();
    assert_eq!(
        template_empty,
        UnixString::try_from(owned_empty.clone()).unwrap()
    );
    let bad_input = "Hi!\0Hello!\0";
    assert!(UnixString::try_from(bad_input).is_err());
}

#[test]
fn can_match_up_to() {
    let haystack = "haystack\0";
    unsafe {
        let needle = "\0";
        assert_eq!(0, haystack.match_up_to(needle.as_ptr()));
        let needle = "h\0";
        assert_eq!(1, haystack.match_up_to(needle.as_ptr()));
        let needle = "haystac\0";
        assert_eq!(7, haystack.match_up_to(needle.as_ptr()));
        let needle = "haystack\0";
        assert_eq!(8, haystack.match_up_to(needle.as_ptr()));
        let needle = "haystack2\0";
        assert_eq!(8, haystack.match_up_to(needle.as_ptr()));
    }
}

#[test]
fn can_check_strlen() {
    unsafe {
        let test = "\0";
        assert_eq!(0, strlen(test.as_ptr()));
        let test = "1\0";
        assert_eq!(1, strlen(test.as_ptr()));
        let test = "10_000_000\0";
        assert_eq!(10, strlen(test.as_ptr()));
    }
}

#[test]
fn can_check_buf_strlen() {
    let test = b"\0";
    assert_eq!(0, buf_strlen(test).unwrap());
    let test = b"1\0";
    assert_eq!(1, buf_strlen(test).unwrap());
    let test = b"10_000_000\0";
    assert_eq!(10, buf_strlen(test).unwrap());
    let bad = b"10_000_000";
    assert!(buf_strlen(bad).is_err());
}
