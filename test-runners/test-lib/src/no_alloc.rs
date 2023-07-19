use tiny_std::io::Read;
use tiny_std::process::{Environment, Stdio};

pub fn spawn_no_args() {
    //
    let mut proc = tiny_std::process::spawn::<0, &str, &str, ()>(
        "/usr/bin/uname\0",
        [],
        &Environment::Inherit,
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        &mut [],
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

pub fn spawn_with_args() {
    let mut proc_with_arg = tiny_std::process::spawn::<3, _, _, ()>(
        "/usr/bin/uname\0",
        ["/usr/bin/uname\0", "-a\0", "\0"],
        &Environment::Inherit,
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        Some(Stdio::MakePipe),
        &mut [],
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
