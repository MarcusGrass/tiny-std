/// Attempts to get the system hostname as a utf8 `String`
/// # Errors
/// Hostname is not utf8
#[cfg(feature = "alloc")]
pub fn host_name() -> Result<alloc::string::String, crate::error::Error> {
    #[allow(unused_imports)]
    use alloc::string::ToString;
    let raw = rusl::unistd::uname()?;
    Ok(raw.nodename()?.to_string())
}
