use crate::string::strlen::{buf_strlen, strlen};

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
