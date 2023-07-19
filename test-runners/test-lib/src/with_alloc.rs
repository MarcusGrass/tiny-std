use alloc::string::String;
use tiny_std::io::Read;
use tiny_std::process::Stdio;

pub(crate) fn spawn_no_args() {
    let mut proc = tiny_std::process::Command::new("/usr/bin/uname")
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
    let mut proc = tiny_std::process::Command::new("/usr/bin/uname")
        .unwrap()
        .arg("-a")
        .unwrap()
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
