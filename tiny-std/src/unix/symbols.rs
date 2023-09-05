#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn _Unwind_Resume() -> ! {
    crate::process::exit(1)
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub extern "C" fn getauxval(_val: u64) -> u64 {
    panic!("This is a bug, the extern `C` function `getauxval` was invoked, we don't use the `C` implementation here, but code using it was generated.")
}

/// Skip lang item feature which will never stabilize and just put the symbol in
/// # Safety
/// Just a symbol that's necessary
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

/// Panic handler
#[panic_handler]
#[cfg(not(feature = "threaded"))]
pub fn on_panic(info: &core::panic::PanicInfo) -> ! {
    unix_print::unix_eprintln!("Main thread panicked: {}", info);
    rusl::process::exit(1)
}
