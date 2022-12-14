use core::ffi::c_void;
use core::hint::unreachable_unchecked;
use core::mem::MaybeUninit;

use sc::syscall;

use rusl::platform::signal::{SigSetT, SIG_DFL, SIG_IGN};
use rusl::platform::{SA_RESTART, SA_RESTORER, SA_SIGINFO};

use crate::error::Error;

/// This struct can differ between architectures, it's the same on aarch64 and `x86_64` though.
#[repr(C)]
struct Sigaction {
    // Fn pointer to an extern C Fn(int) -> ()
    sa_sigaction: SaSigaction,
    sa_flags: i32,
    // Fn pointer to an extern C Fn() -> (), according to the docs shouldn't be used
    // by applications
    sa_restorer: unsafe extern "C" fn() -> !,
    sa_mask: SigSetT,
}

/// On aarch64 and `x86_64` this is a union, depending on if you specify `SA_SIGINFO` you get one or the other.
/// The raw `value` takes either 0 or 1 for `default handling` or `ignore` respectively,
/// Anything other than that or a valid function pointer there is instant UB (likely a segfault)
#[repr(C)]
#[derive(Copy, Clone)]
union SaSigaction {
    value: usize,
    sa_handler: unsafe extern "C" fn(i32),
    sa_sigaction: unsafe extern "C" fn(i32, info: *mut SigInfo, *const c_void),
}

#[derive(Debug, Copy, Clone)]
pub enum SaSignalaction {
    Dfl,
    Ign,
    Handler(unsafe extern "C" fn(i32)),
    SigAction(unsafe extern "C" fn(i32, info: *mut SigInfo, *const c_void)),
}

#[repr(C)]
pub struct SigInfo {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    pub si_trapno: i32,
    pub si_pid: i32,
    pub _pad: [i32; 27],
    pub _align: [u64; 0],
}

/// We don't have to do much here, just return from the handler
unsafe extern "C" fn restorer() -> ! {
    syscall!(RT_SIGRETURN);
    unreachable_unchecked()
}

/// Some (not all) signals it makes sense to handle.
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man7/signal.7.html)
/// Some are expclicitly disallowed (`SigKill`, `SigStop`), some cannot be handled safely by this
/// library yet `SigAbrt` f.e.
pub enum CatchSignal {
    // Keyboard interrupt
    SIGINT,
    // Termination signal
    SIGTERM,
    // Hangup
    SIGHUP,
    // Invalid memory reference
    SIGSEGV,
    // Child stopped or terminated
    SIGCHLD,
}

impl CatchSignal {
    fn into_raw(self) -> i32 {
        match self {
            CatchSignal::SIGINT => rusl::platform::SIGINT,
            CatchSignal::SIGTERM => rusl::platform::SIGTERM,
            CatchSignal::SIGHUP => rusl::platform::SIGHUP,
            CatchSignal::SIGSEGV => rusl::platform::SIGSEGV,
            CatchSignal::SIGCHLD => rusl::platform::SIGCHLD,
        }
    }
}

/// Attempts to set up a signal handler for the provided signal number
/// # Errors
/// Syscall errors if the provided functions doesn't make sense or the `syscall` doesn't make sense.
/// # Safety
/// Invalid function pointers is UB.
/// Additionally, signal handlers have to by async-signal-safe. Essentially meaning that
/// anything they touch have to be safely accessible concurrently. Some things `Rust` may guarantee
/// but many it won't.
pub unsafe fn add_signal_action(
    signal: CatchSignal,
    sigaction: SaSignalaction,
) -> crate::error::Result<()> {
    let mut constructed_action: MaybeUninit<Sigaction> = MaybeUninit::uninit();
    let mut flags = SA_RESTART | SA_RESTORER;
    let s_ptr = constructed_action.as_mut_ptr();
    (*s_ptr).sa_mask = SigSetT::default();
    (*s_ptr).sa_restorer = restorer;
    (*s_ptr).sa_sigaction = match sigaction {
        SaSignalaction::Dfl => SaSigaction { value: SIG_DFL },
        SaSignalaction::Ign => SaSigaction { value: SIG_IGN },
        SaSignalaction::Handler(sa_handler) => {
            // TODO: Double check this
            flags -= SA_SIGINFO;
            SaSigaction { sa_handler }
        }
        SaSignalaction::SigAction(sa_sigaction) => {
            flags |= SA_SIGINFO;
            SaSigaction { sa_sigaction }
        }
    };
    (*s_ptr).sa_flags = flags;
    let res = syscall!(
        RT_SIGACTION,
        signal.into_raw(),
        constructed_action.as_ptr(),
        0,
        8
    ) as isize;
    if res < 0 {
        return Err(Error::Os {
            msg: "Setting sigaction failed",
            code: res as i32,
        });
    }

    Ok(())
}
