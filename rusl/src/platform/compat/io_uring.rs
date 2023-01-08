use core::ptr::NonNull;
use core::sync::atomic::AtomicU32;

use linux_rust_bindings::io_uring::{
    io_cqring_offsets, io_sqring_offsets, io_uring_cqe, io_uring_params, io_uring_sqe,
};

use crate::platform::Fd;

transparent_bitflags! {
    pub struct IoUringParamFlags: u32 {
        const IORING_SETUP_IOPOLL = linux_rust_bindings::io_uring::IORING_SETUP_IOPOLL as u32;
        const IORING_SETUP_SQPOLL = linux_rust_bindings::io_uring::IORING_SETUP_SQPOLL as u32;
        const IORING_SETUP_SQ_AFF = linux_rust_bindings::io_uring::IORING_SETUP_SQ_AFF as u32;
        const IORING_SETUP_CQSIZE = linux_rust_bindings::io_uring::IORING_SETUP_CQSIZE as u32;
        const IORING_SETUP_CLAMP = linux_rust_bindings::io_uring::IORING_SETUP_CLAMP as u32;
        const IORING_SETUP_ATTACH_WQ = linux_rust_bindings::io_uring::IORING_SETUP_ATTACH_WQ as u32;
        const IORING_SETUP_R_DISABLED = linux_rust_bindings::io_uring::IORING_SETUP_R_DISABLED as u32;
        const IORING_SETUP_SUBMIT_ALL = linux_rust_bindings::io_uring::IORING_SETUP_SUBMIT_ALL as u32;
        const IORING_SETUP_COOP_TASKRUN = linux_rust_bindings::io_uring::IORING_SETUP_COOP_TASKRUN as u32;
        const IORING_SETUP_TASKRUN_FLAG = linux_rust_bindings::io_uring::IORING_SETUP_TASKRUN_FLAG as u32;
        const IORING_SETUP_SQE128 = linux_rust_bindings::io_uring::IORING_SETUP_SQE128 as u32;
        const IORING_SETUP_CQE32 = linux_rust_bindings::io_uring::IORING_SETUP_CQE32 as u32;
        const IORING_SETUP_SINGLE_ISSUER = linux_rust_bindings::io_uring::IORING_SETUP_SINGLE_ISSUER as u32;
        const IORING_SETUP_DEFER_TASKRUN = linux_rust_bindings::io_uring::IORING_SETUP_DEFER_TASKRUN as u32;
    }
}

#[repr(transparent)]
pub struct IoUringSubmissionQueueEntry(pub io_uring_sqe);

#[repr(transparent)]
pub struct IoUringCompletionQueueEntry(pub io_uring_cqe);

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct IoUringParams(pub io_uring_params);

impl IoUringParams {
    /// Fields are populated by the kernel
    #[must_use]
    pub const fn new(flags: IoUringParamFlags) -> Self {
        Self(io_uring_params {
            sq_entries: 0,
            cq_entries: 0,
            flags: flags.bits(),
            sq_thread_cpu: 0,
            sq_thread_idle: 0,
            features: 0,
            wq_fd: 0,
            resv: [0u32; 3usize],
            sq_off: io_sqring_offsets {
                head: 0,
                tail: 0,
                ring_mask: 0,
                ring_entries: 0,
                flags: 0,
                dropped: 0,
                array: 0,
                resv1: 0,
                resv2: 0,
            },
            cq_off: io_cqring_offsets {
                head: 0,
                tail: 0,
                ring_mask: 0,
                ring_entries: 0,
                overflow: 0,
                cqes: 0,
                flags: 0,
                resv1: 0,
                resv2: 0,
            },
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct IoUring {
    pub fd: Fd,
    pub(crate) sq: UringSubmissionQueue,
    pub(crate) cq: UringCompletionQueue,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct UringSubmissionQueue {
    pub(crate) ring_size: usize,
    pub(crate) ring_ptr: usize,
    pub(crate) kernel_head: NonNull<AtomicU32>,
    pub(crate) kernel_tail: NonNull<AtomicU32>,
    pub(crate) kernel_flags: NonNull<AtomicU32>,
    pub(crate) kernel_dropped: NonNull<AtomicU32>,
    pub(crate) kernel_array: NonNull<AtomicU32>,
    pub(crate) head: usize,
    pub(crate) tail: usize,
    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,
    pub(crate) entries: NonNull<IoUringSubmissionQueueEntry>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct UringCompletionQueue {
    pub(crate) ring_size: usize,
    pub(crate) ring_ptr: usize,
    pub(crate) kernel_head: NonNull<AtomicU32>,
    pub(crate) kernel_tail: NonNull<AtomicU32>,
    pub(crate) kernel_flags: NonNull<AtomicU32>,
    pub(crate) kernel_overflow: NonNull<AtomicU32>,
    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,
    pub(crate) entries: NonNull<IoUringCompletionQueueEntry>,
}
