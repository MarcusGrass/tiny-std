use core::mem::MaybeUninit;

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
            ],
        }
    }
}

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
