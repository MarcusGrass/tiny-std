use core::mem::MaybeUninit;

pub const SIGSET_LEN: usize = 128 / core::mem::size_of::<usize>();
/// A set of signals
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SigSetT {
    __val: [MaybeUninit<u64>; 16],
}

impl Default for SigSetT {
    fn default() -> Self {
        Self {
            __val: [
                MaybeUninit::new(0),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
            ]
        }
    }
}

pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGILL: i32 = 4;
pub const SIGTRAP: i32 = 5;
pub const SIGABRT: i32 = 6;
pub const SIGBUS: i32 = 7;
pub const SIGFPE: i32 = 8;
pub const SIGKILL: i32 = 9;
pub const SIGUSR1: i32 = 10;
pub const SIGSEGV: i32 = 11;
pub const SIGUSR2: i32 = 12;
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
pub const SIGTERM: i32 = 15;
pub const SIGCHLD: i32 = 17;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;

pub const SA_NODEFER: i32 = 0x40000000;
#[allow(overflowing_literals)]
pub const SA_RESETHAND: i32 = 0x80000000;
pub const SA_RESTART: i32 = 0x10000000;
pub const SA_NOCLDSTOP: i32 = 0x00000001;

// These signals can differ but are the same on x86_64 and aarch64
pub const SA_ONSTACK: i32 = 0x08000000;
pub const SA_SIGINFO: i32 = 0x00000004;
pub const SA_NOCLDWAIT: i32 = 0x00000002;
pub const SA_RESTORER: i32 = 0x04000000;