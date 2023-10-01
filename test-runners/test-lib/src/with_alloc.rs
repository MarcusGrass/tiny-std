use alloc::string::String;
use tiny_std::io::Read;
use tiny_std::process::Stdio;
use tiny_std::UnixStr;

pub(crate) fn spawn_no_args() {
    let mut proc =
        tiny_std::process::Command::new(UnixStr::try_from_str("/usr/bin/uname\0").unwrap())
            .unwrap()
            .stdout(Stdio::MakePipe)
            .spawn()
            .unwrap();
    let exit = proc.wait().unwrap();
    assert_eq!(0, exit);
    let mut out = proc.stdout.unwrap();
    let mut string = String::new();
    out.read_to_string(&mut string).unwrap();
    assert_eq!("Linux\n", string,);
}

pub(crate) fn spawn_with_args() {
    let mut proc =
        tiny_std::process::Command::new(UnixStr::try_from_str("/usr/bin/uname\0").unwrap())
            .unwrap()
            .arg(UnixStr::try_from_str("-a\0").unwrap())
            .stdout(Stdio::MakePipe)
            .spawn()
            .unwrap();
    let exit = proc.wait().unwrap();
    assert_eq!(0, exit);
    let mut out = proc.stdout.unwrap();
    let mut string = String::new();
    out.read_to_string(&mut string).unwrap();
    assert!(string.starts_with("Linux"));
    assert!(string.len() > 64);
}
