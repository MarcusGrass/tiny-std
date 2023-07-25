use crate::platform::numbers::NonNegativeI32;
use core::mem::MaybeUninit;

#[derive(Debug, Copy, Clone)]
pub struct SignalKind(pub(crate) NonNegativeI32);

impl SignalKind {
    pub const SIGHUP: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGHUP,
    ));
    pub const SIGINT: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGINT,
    ));
    pub const SIGQUIT: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGQUIT,
    ));
    pub const SIGILL: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGILL,
    ));
    pub const SIGTRAP: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGTRAP,
    ));
    pub const SIGABRT: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGABRT,
    ));
    pub const SIGIOT: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGIOT,
    ));
    pub const SIGBUS: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGBUS,
    ));
    pub const SIGFPE: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGFPE,
    ));
    pub const SIGKILL: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGKILL,
    ));
    pub const SIGUSR1: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGUSR1,
    ));
    pub const SIGSEGV: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGSEGV,
    ));
    pub const SIGUSR2: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGUSR2,
    ));
    pub const SIGPIPE: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGPIPE,
    ));
    pub const SIGALRM: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGALRM,
    ));
    pub const SIGTERM: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGTERM,
    ));
    pub const SIGSTKFLT: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGSTKFLT,
    ));
    pub const SIGCHLD: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGCHLD,
    ));
    pub const SIGCONT: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGCONT,
    ));
    pub const SIGSTOP: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGSTOP,
    ));
    pub const SIGTSTP: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGTSTP,
    ));
    pub const SIGTTIN: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGTTIN,
    ));
    pub const SIGTTOU: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGTTOU,
    ));
    pub const SIGURG: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGURG,
    ));
    pub const SIGXCPU: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGXCPU,
    ));
    pub const SIGXFSZ: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGXFSZ,
    ));
    pub const SIGVTALRM: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGVTALRM,
    ));
    pub const SIGPROF: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGPROF,
    ));
    pub const SIGWINCH: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGWINCH,
    ));
    pub const SIGIO: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGIO,
    ));
    pub const SIGPOLL: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGPOLL,
    ));
    pub const SIGPWR: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGPWR,
    ));
    pub const SIGSYS: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGSYS,
    ));
    pub const SIGUNUSED: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGUNUSED,
    ));
    pub const SIGRTMIN: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGRTMIN,
    ));
    pub const SIGSTKSZ: Self = Self(NonNegativeI32::comptime_checked_new(
        linux_rust_bindings::signal::SIGSTKSZ,
    ));
}

transparent_bitflags! {
    pub struct SaMask: i64 {
        const DEFAULT = 0;
        const SA_NOCLDSTOP = linux_rust_bindings::signal::SA_NOCLDSTOP as i64;
        const SA_NOCLDWAIT = linux_rust_bindings::signal::SA_NOCLDWAIT as i64;
        const SA_SIGINFO = linux_rust_bindings::signal::SA_SIGINFO as i64;
        const SA_UNSUPPORTED = linux_rust_bindings::signal::SA_UNSUPPORTED as i64;
        const SA_EXPOSE_TAGBITS = linux_rust_bindings::signal::SA_EXPOSE_TAGBITS as i64;
        const SA_ONSTACK = linux_rust_bindings::signal::SA_ONSTACK as i64;
        const SA_RESTART = linux_rust_bindings::signal::SA_RESTART as i64;
        const SA_NODEFER = linux_rust_bindings::signal::SA_NODEFER as i64;
        const SA_RESETHAND = linux_rust_bindings::signal::SA_RESETHAND;
        const SA_NOMASK = linux_rust_bindings::signal::SA_NOMASK as i64;
        const SA_ONESHOT = linux_rust_bindings::signal::SA_ONESHOT;
        const SA_RESTORER = linux_rust_bindings::signal::SA_RESTORER as i64;
    }
}
/// A set of signals
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SigSetT {
    __val: [MaybeUninit<u64>; 16],
}

impl SigSetT {
    #[must_use]
    pub fn new() -> Self {
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
            ],
        }
    }
}

impl Default for SigSetT {
    fn default() -> Self {
        Self::new()
    }
}

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
