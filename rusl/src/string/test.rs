use crate::string::unix_str::{AsUnixStr};

#[test]
#[cfg(not(feature = "alloc"))]
fn errs_on_not_null_without_alloc() {
    let use_with_non_null_without_allocator = "Hello".exec_with_self_as_ptr(|_| {
        Ok(())
    });
    assert!(use_with_non_null_without_allocator.is_err());
}

#[test]
#[cfg(feature = "alloc")]
fn accepts_not_null_on_alloc() {
    let use_with_non_null_without_allocator = "Hello".exec_with_self_as_ptr(|_| {
        Ok(())
    });
    assert!(use_with_non_null_without_allocator.is_ok());
}

#[test]
fn errs_on_many_null() {
    let use_with_non_null_without_allocator = "Hello\0oh no\0".exec_with_self_as_ptr(|_| {
        Ok(())
    });
    assert!(use_with_non_null_without_allocator.is_err());
}