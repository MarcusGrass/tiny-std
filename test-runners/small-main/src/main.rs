#![no_std]
#![no_main]
#![allow(dead_code)]

use rusl::platform::vdso::{AT_GID, AT_PAGESZ, AT_RANDOM, AT_SYSINFO_EHDR, AT_UID};
use tiny_std::io::Read;
use tiny_std::process::{Environment, Stdio};

/// Panic handler
#[panic_handler]
pub fn on_panic(info: &core::panic::PanicInfo) -> ! {
    unix_print::unix_eprintln!("Panicked {info}");
    tiny_std::process::exit(1)
}

#[no_mangle]
pub fn main() -> i32 {
    let v = tiny_std::env::var("HOME").unwrap();
    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!("/home/gramar", v);
    }
    #[cfg(target_arch = "aarch64")]
    {
        // Running this in a container
        assert_eq!("/root", v);
    }
    let mut args = tiny_std::env::args();
    args.next();
    let arg = args.next().unwrap().unwrap();
    assert_eq!("dummy_arg", arg);
    let mut os_args = tiny_std::env::args_os();
    let os_arg = os_args.next().unwrap();
    assert_eq!("dummy_arg", os_arg.as_str().unwrap());
    test_spawn_no_args();
    test_spawn_with_args();
    unsafe { test_aux_values() };
    test_time();
    0
}

#[no_mangle]
fn test_spawn_no_args() {
    //
    let mut proc = tiny_std::process::spawn::<0, &str, &str>(
        "/usr/bin/uname\0",
        [],
        Environment::Inherit,
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let exit = proc.wait().unwrap();
    assert_eq!(0, exit);
    let mut out = proc.stdout.unwrap();
    let mut bytes = [0u8; 64];
    let read_bytes = out.read(&mut bytes).unwrap();
    assert_eq!(
        "Linux\n",
        core::str::from_utf8(&bytes[..read_bytes]).unwrap()
    );
}

#[no_mangle]
fn test_spawn_with_args() {
    let mut proc_with_arg = tiny_std::process::spawn(
        "/usr/bin/uname\0",
        ["/usr/bin/uname\0", "-a\0", "\0"],
        Environment::Inherit,
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let exit = proc_with_arg.wait().unwrap();
    let mut err = proc_with_arg.stdout.unwrap();
    let mut bytes = [0u8; 256];
    let read_bytes = err.read(&mut bytes).unwrap();
    let content = core::str::from_utf8(&bytes[..read_bytes]).unwrap();
    assert!(content.starts_with("Linux"));
    assert!(content.len() > 64);
    assert_eq!(0, exit);
}

unsafe fn test_aux_values() {
    let page_size = tiny_std::start::get_aux_value(AT_PAGESZ).unwrap();
    assert_eq!(4096, page_size);
    // 16 random bytes
    let random = tiny_std::start::get_aux_value(AT_RANDOM).unwrap() as *const u128;
    let _val = random.read();
    let uid = tiny_std::start::get_aux_value(AT_UID).unwrap();
    let gid = tiny_std::start::get_aux_value(AT_GID).unwrap();
    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!(1000, uid);
        assert_eq!(1000, gid);
    }
    // Can only run this in a docker container at the moment
    #[cfg(target_arch = "aarch64")]
    {
        assert_eq!(0, uid);
        assert_eq!(0, gid);
    }
    // TODO: Fix VDSO for aarch64
    #[cfg(not(target_arch = "aarch64"))]
    let _vdso = tiny_std::start::get_aux_value(rusl::platform::vdso::AT_SYSINFO_EHDR).unwrap();
}

fn test_time() {
    let now = tiny_std::time::Instant::now();
    let later = tiny_std::time::Instant::now();
    assert!(later > now);
    let now = tiny_std::time::SystemTime::now();
    let later = tiny_std::time::SystemTime::now();
    assert!(later > now);
}
