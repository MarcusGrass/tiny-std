#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::hint::unreachable_unchecked;

#[cfg(feature = "alloc")]
use rusl::string::unix_str::UnixString;
use rusl::string::unix_str::{AsUnixStr, UnixStr};
use rusl::platform::EINTR;
use rusl::platform::WNOHANG;
use rusl::platform::{GidT, PidT, UidT};
use rusl::platform::{STDERR, STDIN, STDOUT};
use rusl::unistd::OpenFlags;

use crate::error::{Error, Result};
use crate::fs::OpenOptions;
use crate::io::{Read, Write};
use crate::unix::fd::{OwnedFd, RawFd};

const DEV_NULL: &str = "/dev/null\0";

/// Terminates this process
#[inline]
pub fn exit(code: i32) -> ! {
    rusl::process::exit(code)
}

#[cfg(feature = "alloc")]
pub struct Command {
    bin: UnixString,
    args: Vec<UnixString>,
    argv: Argv,
    env: Environment,
    cwd: Option<UnixString>,
    uid: Option<UidT>,
    gid: Option<GidT>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    pgroup: Option<PidT>,
}

// Create a new type for argv, so that we can make it `Send` and `Sync`
#[cfg(feature = "alloc")]
struct Argv(Vec<*const u8>);

// It is safe to make `Argv` `Send` and `Sync`, because it contains
// pointers to memory owned by `Command.args`
#[cfg(feature = "alloc")]
unsafe impl Send for Argv {}

#[cfg(feature = "alloc")]
unsafe impl Sync for Argv {}

// Create a new type for argv, so that we can make it `Send` and `Sync`
#[cfg(feature = "alloc")]
struct Envp(Vec<*const u8>);

// It is safe to make `Argv` `Send` and `Sync`, because it contains
// pointers to memory owned by `Command.args`
#[cfg(feature = "alloc")]
unsafe impl Send for Envp {}

#[cfg(feature = "alloc")]
unsafe impl Sync for Envp {}

#[cfg(feature = "alloc")]
impl Command {
    /// Constructs a new command, setting the first argument as the binary's name
    /// # Errors
    /// If the string is not `C string compatible`
    pub fn new<A: AsUnixStr>(bin: A) -> Result<Self> {
        let bin = bin.to_unix_string()?;
        let bin_ptr = bin.as_ptr();
        Ok(Self {
            bin: bin.clone(),
            args: vec![bin],
            argv: Argv(vec![bin_ptr, core::ptr::null()]),
            env: Environment::default(),
            cwd: None,
            uid: None,
            gid: None,
            stdin: None,
            stdout: None,
            stderr: None,
            pgroup: None,
        })
    }

    /// # Errors
    /// If the string is not `C string compatible`
    pub fn env<A: AsUnixStr>(&mut self, env: A) -> Result<&mut Self> {
        #[cfg(feature = "start")]
        if !matches!(self.env, Environment::Inherit | Environment::None) {
            self.env = Environment::Provided(ProvidedEnvironment {
                vars: vec![],
                envp: Envp(vec![core::ptr::null()]),
            })
        };
        #[cfg(not(feature = "start"))]
        if !matches!(self.env, Environment::None) {
            self.env = Environment::Provided(ProvidedEnvironment {
                vars: vec![],
                envp: Envp(vec![core::ptr::null()]),
            });
        };
        if let Environment::Provided(pe) = &mut self.env {
            let s = env.to_unix_string()?;
            pe.envp.0[pe.vars.len()] = s.as_ptr();
            pe.envp.0.push(core::ptr::null());
            pe.vars.push(s);
        }
        Ok(self)
    }

    /// # Errors
    /// If the string is not `C string compatible`
    pub fn envs<A: AsUnixStr>(&mut self, envs: Vec<A>) -> Result<&mut Self> {
        for env in envs {
            self.env(env)?;
        }
        Ok(self)
    }

    /// # Errors
    /// If the string is not `C string compatible`
    pub fn arg<A: AsUnixStr>(&mut self, arg: A) -> Result<&mut Self> {
        let unix_string = arg.to_unix_string()?;
        self.argv.0[self.args.len()] = unix_string.as_ptr();
        self.argv.0.push(core::ptr::null());
        self.args.push(unix_string);
        Ok(self)
    }

    /// # Errors
    /// If the string is not `C string compatible`
    pub fn args<A: AsUnixStr>(&mut self, args: &[A]) -> Result<&mut Self> {
        self.args.reserve(args.len());
        self.argv.0.reserve(args.len());
        for arg in args {
            self.arg(arg)?;
        }
        Ok(self)
    }

    /// # Errors
    /// If the string is not `C string compatible`
    pub fn cwd<A: AsUnixStr>(&mut self, dir: A) -> Result<&mut Self> {
        self.cwd = Some(dir.to_unix_string()?);
        Ok(self)
    }

    pub fn uid(&mut self, id: UidT) -> &mut Self {
        self.uid = Some(id);
        self
    }

    pub fn gid(&mut self, id: GidT) -> &mut Self {
        self.gid = Some(id);
        self
    }

    pub fn pgroup(&mut self, pgroup: PidT) -> &mut Self {
        self.pgroup = Some(pgroup);
        self
    }

    pub fn stdin(&mut self, stdin: Stdio) -> &mut Self {
        self.stdin = Some(stdin);
        self
    }

    pub fn stdout(&mut self, stdout: Stdio) -> &mut Self {
        self.stdin = Some(stdout);
        self
    }

    pub fn stderr(&mut self, stderr: Stdio) -> &mut Self {
        self.stdin = Some(stderr);
        self
    }

    /// Spawns a new child process from this command.
    /// # Errors
    /// See `spawn`
    pub fn spawn(&self) -> Result<Child> {
        const NULL_ENV: [*const u8; 1] = [core::ptr::null()];
        let envp = match &self.env {
            #[cfg(feature = "start")]
            Environment::Inherit => unsafe { crate::start::ENV.env_p },
            Environment::None => NULL_ENV.as_ptr(),
            Environment::Provided(provided) => provided.envp.0.as_ptr(),
        };
        unsafe {
            do_spawn(
                &self.bin,
                self.argv.0.as_ptr(),
                envp,
                Stdio::Inherit,
                true,
                self.stdin,
                self.stdout,
                self.stderr,
                self.cwd.as_deref(),
                self.uid,
                self.gid,
                self.pgroup,
            )
        }
    }
}

pub struct Child {
    pub(crate) handle: Process,

    pub stdin: Option<AnonPipe>,

    pub stdout: Option<AnonPipe>,

    pub stderr: Option<AnonPipe>,
}

impl Child {
    /// Get the backing pid of this Child
    #[inline]
    #[must_use]
    pub fn get_pid(&self) -> i32 {
        self.handle.pid
    }
    /// Waits for this child process to finish retuning its exit code
    /// # Errors
    /// Os errors relating to waiting for process
    #[inline]
    pub fn wait(&mut self) -> Result<i32> {
        drop(self.stdin.take());
        self.handle.wait()
    }

    /// Attempts to wait for this child process to finish, returns Ok(None) if
    /// child still hasn't finished, otherwise returns the exit code
    /// # Errors
    /// Os errors relating to waiting for process
    #[inline]
    pub fn try_wait(&mut self) -> Result<Option<i32>> {
        self.handle.try_wait()
    }
}

pub struct Process {
    pid: i32,
    status: Option<i32>,
}

impl Process {
    fn wait(&mut self) -> Result<i32> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        let res = rusl::process::wait_pid(self.pid, 0)?;
        self.status = Some(res.status);
        Ok(res.status)
    }

    fn try_wait(&mut self) -> Result<Option<i32>> {
        if let Some(status) = self.status {
            return Ok(Some(status));
        }
        let res = rusl::process::wait_pid(self.pid, WNOHANG)?;
        if res.pid == 0 {
            Ok(None)
        } else {
            self.status = Some(res.status);
            Ok(Some(res.status))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
}

impl Stdio {
    fn to_child_stdio(self, readable: bool) -> Result<(ChildStdio, Option<AnonPipe>)> {
        match self {
            Stdio::Inherit => Ok((ChildStdio::Inherit, None)),

            Stdio::MakePipe => {
                let pipe = rusl::unistd::pipe2(OpenFlags::O_CLOEXEC)?;
                let (ours, theirs) = if readable {
                    (pipe.out_pipe, pipe.in_pipe)
                } else {
                    (pipe.in_pipe, pipe.out_pipe)
                };
                Ok((
                    ChildStdio::Owned(OwnedFd(theirs)),
                    Some(AnonPipe(OwnedFd(ours))),
                ))
            }

            Stdio::Null => {
                let mut opts = OpenOptions::new();
                opts.read(readable);
                opts.write(!readable);
                let fd = opts.open(DEV_NULL)?;
                Ok((ChildStdio::Owned(fd.into_inner()), None))
            }
        }
    }
}

pub enum ChildStdio {
    Inherit,
    Owned(OwnedFd),
}

impl ChildStdio {
    fn fd(&self) -> Option<RawFd> {
        match self {
            ChildStdio::Inherit => None,
            ChildStdio::Owned(fd) => Some(fd.0),
        }
    }
}

#[non_exhaustive]
pub enum Environment {
    #[cfg(feature = "start")]
    Inherit,
    None,
    #[cfg(feature = "alloc")]
    Provided(ProvidedEnvironment),
}

#[cfg(feature = "alloc")]
pub struct ProvidedEnvironment {
    vars: Vec<UnixString>,
    envp: Envp,
}

impl Default for Environment {
    fn default() -> Self {
        #[cfg(feature = "start")]
        {
            Environment::Inherit
        }
        #[cfg(not(feature = "start"))]
        {
            Environment::None
        }
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
unsafe fn do_spawn(
    bin: &UnixStr,
    argv: *const *const u8,
    envp: *const *const u8,
    default_stdio: Stdio,
    needs_stdin: bool,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    cwd: Option<&UnixStr>,
    uid: Option<UidT>,
    gid: Option<GidT>,
    pgroup: Option<PidT>,
) -> Result<Child> {
    const CLOEXEC_MSG_FOOTER: [u8; 4] = *b"NOEX";
    let (ours, theirs) = setup_io(default_stdio, needs_stdin, stdin, stdout, stderr)?;
    let sync_pipe = rusl::unistd::pipe2(OpenFlags::O_CLOEXEC)?;
    let (read_pipe, write_pipe) = (sync_pipe.in_pipe, sync_pipe.out_pipe);
    let child_pid = rusl::process::fork()?;
    // From this point we're two processes
    if child_pid == 0 {
        // Executing as child process
        let _ = rusl::unistd::close(read_pipe);
        if let Some(fd) = theirs.stdin.fd() {
            rusl::unistd::dup2(fd, STDIN)?;
        }
        if let Some(fd) = theirs.stdout.fd() {
            rusl::unistd::dup2(fd, STDOUT)?;
        }
        if let Some(fd) = theirs.stderr.fd() {
            rusl::unistd::dup2(fd, STDERR)?;
        }
        if let Some(cwd) = cwd {
            rusl::unistd::chdir(cwd)?;
        }
        if let Some(uid) = uid {
            rusl::unistd::setuid(uid)?;
        }
        if let Some(gid) = gid {
            rusl::unistd::setgid(gid)?;
        }
        if let Some(pgroup) = pgroup {
            rusl::unistd::setpgid(0, pgroup)?;
        }
        let e = if let Err(e) = rusl::process::execve(bin, argv, envp) {
            e
        } else {
            unreachable_unchecked();
        };
        let code: [u8; 4] = if let Some(code) = e.code {
            code.to_be_bytes()
        } else {
            rusl::process::exit(1)
        };
        let bytes = [
            code[0],
            code[1],
            code[2],
            code[3],
            CLOEXEC_MSG_FOOTER[0],
            CLOEXEC_MSG_FOOTER[1],
            CLOEXEC_MSG_FOOTER[2],
            CLOEXEC_MSG_FOOTER[3],
        ];
        let _ = rusl::unistd::write(write_pipe, &bytes);
        rusl::process::exit(1);
    }
    let _ = rusl::unistd::close(write_pipe);
    let mut process = Process {
        pid: child_pid as i32,
        status: None,
    };
    let mut bytes = [0, 0, 0, 0, 0, 0, 0, 0];
    loop {
        match rusl::unistd::read(read_pipe, &mut bytes) {
            Ok(0) => {
                let child = Child {
                    handle: process,
                    stdin: ours.stdin,
                    stdout: ours.stdout,
                    stderr: ours.stderr,
                };
                return Ok(child);
            }
            Ok(8) => {
                let (errno, footer) = bytes.split_at(4);
                if CLOEXEC_MSG_FOOTER != footer {
                    return Err(Error::no_code("Validation on the CLOEXEC pipe failed"));
                }

                let errno = i32::from_be_bytes(errno.try_into().unwrap_unchecked());
                process.wait()?;
                return Err(Error::os("Failed to wait for process", errno));
            }
            Err(ref e) if matches!(e.code, Some(EINTR)) => {}
            Err(_) => {
                process.wait()?;
                return Err(Error::no_code("The cloexec pipe failed"));
            }
            Ok(..) => {
                // pipe I/O up to PIPE_BUF bytes should be atomic
                process.wait()?;
                return Err(Error::no_code("short read on the CLOEXEC pipe"));
            }
        }
    }
}

/// Spawns a process with the provided arguments. On no arguments, the binary will be set as the first
/// argument as per best practice, on any args, it's up to the caller to follow that best practice or not.
/// `arg_v` must null terminated, since there is currently no way to do constant ops, ie
/// put an array of length `N + 1` on the stack, the last value is discarded
/// # Errors
/// OS errors relating to permission on the binary, as well as other errors relating to pipe creation
/// and process spawning.
/// # Notes
/// We have to do some gating here, since we're copying the pointer out of the closure it'd dangle
/// after if we did an allocation before that closure.
#[cfg(not(feature = "alloc"))]
#[allow(clippy::too_many_arguments)]
pub fn spawn<const N: usize, BIN: AsUnixStr, ARG: AsUnixStr>(
    bin: BIN,
    argv: [ARG; N],
    env: Environment,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    cwd: Option<&UnixStr>,
    uid: Option<UidT>,
    gid: Option<GidT>,
    pgroup: Option<PidT>,
) -> Result<Child> {
    const NO_ENV: [*const u8; 1] = [core::ptr::null()];
    let mut no_args: [*const u8; 2] = [core::ptr::null_mut(), core::ptr::null_mut()];
    let envp = match env {
        #[cfg(feature = "start")]
        Environment::Inherit => unsafe { crate::start::ENV.env_p },
        Environment::None => NO_ENV.as_ptr(),
    };
    let mut new_args = [core::ptr::null(); N];
    let arg_ptr = if argv.is_empty() {
        // Make sure we at least send the bin as arg
        bin.exec_with_self_as_ptr(|ptr| {
            no_args[0] = ptr;
            Ok(())
        })?;
        no_args.as_ptr()
    } else {
        for (ind, arg) in argv.into_iter().enumerate() {
            arg.exec_with_self_as_ptr(|ptr| {
                new_args[ind] = ptr;
                Ok(())
            })?;
        }
        new_args[N - 1] = core::ptr::null();
        new_args.as_ptr()
    };
    // Only safe to do on no-alloc, since we may create a string there and the pointer will
    // dangle if we take it out of the closure
    let bin_ptr = bin.exec_with_self_as_ptr(Ok)?;
    let bin_str = unsafe { UnixStr::from_ptr(bin_ptr) };
    unsafe {
        do_spawn(
            bin_str,
            arg_ptr,
            envp,
            Stdio::Inherit,
            true,
            stdin,
            stdout,
            stderr,
            cwd,
            uid,
            gid,
            pgroup,
        )
    }
}

pub struct AnonPipe(OwnedFd);

impl Read for AnonPipe {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(rusl::unistd::read(self.0 .0, buf)?)
    }
}

impl Write for AnonPipe {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(rusl::unistd::write(self.0 .0, buf)?)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// passed back to std::process with the pipes connected to the child, if any
// were requested
pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}

// passed to do_exec() with configuration of what the child stdio should look
// like
pub struct ChildPipes {
    pub stdin: ChildStdio,
    pub stdout: ChildStdio,
    pub stderr: ChildStdio,
}

fn setup_io(
    default: Stdio,
    needs_stdin: bool,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
) -> Result<(StdioPipes, ChildPipes)> {
    let null = Stdio::Null;
    let default_stdin = if needs_stdin { default } else { null };
    let stdin = stdin.unwrap_or(default_stdin);
    let stdout = stdout.unwrap_or(default);
    let stderr = stderr.unwrap_or(default);
    let (their_stdin, our_stdin) = stdin.to_child_stdio(true)?;
    let (their_stdout, our_stdout) = stdout.to_child_stdio(false)?;
    let (their_stderr, our_stderr) = stderr.to_child_stdio(false)?;
    let ours = StdioPipes {
        stdin: our_stdin,
        stdout: our_stdout,
        stderr: our_stderr,
    };
    let theirs = ChildPipes {
        stdin: their_stdin,
        stdout: their_stdout,
        stderr: their_stderr,
    };
    Ok((ours, theirs))
}
