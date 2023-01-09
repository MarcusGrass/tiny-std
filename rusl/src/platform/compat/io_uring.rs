use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};

use linux_rust_bindings::io_uring::{
    __BindgenUnionField, io_cqring_offsets, io_sqring_offsets, io_uring_cqe, io_uring_params,
    io_uring_sqe, io_uring_sqe__bindgen_ty_1, io_uring_sqe__bindgen_ty_2,
    io_uring_sqe__bindgen_ty_3, io_uring_sqe__bindgen_ty_4, io_uring_sqe__bindgen_ty_5,
    io_uring_sqe__bindgen_ty_6,
};

use crate::platform::{Fd, Mode, OpenFlags, AT_FDCWD};
use crate::string::unix_str::UnixStr;

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

transparent_bitflags! {
    pub struct IoUringSQEFlags: u8 {
        const IOSQE_FIXED_FILE = linux_rust_bindings::io_uring::IOSQE_FIXED_FILE_BIT as u8;
        const IOSQE_IO_DRAIN = linux_rust_bindings::io_uring::IOSQE_IO_DRAIN_BIT as u8;
        const IOSQE_IO_LINK = linux_rust_bindings::io_uring::IOSQE_IO_LINK_BIT as u8;
        const IOSQE_IO_HARDLINK = linux_rust_bindings::io_uring::IOSQE_IO_HARDLINK_BIT as u8;
        const IOSQE_ASYNC = linux_rust_bindings::io_uring::IOSQE_ASYNC_BIT as u8;
        const IOSQE_BUFFER_SELECT = linux_rust_bindings::io_uring::IOSQE_BUFFER_SELECT_BIT as u8;
        const IOSQE_CQE_SKIP_SUCCESS = linux_rust_bindings::io_uring::IOSQE_CQE_SKIP_SUCCESS_BIT as u8;
    }
}

transparent_bitflags! {
    pub struct IoUringEnterFlags: u32 {
        const IORING_ENTER_GETEVENTS = linux_rust_bindings::io_uring::IORING_ENTER_GETEVENTS as u32;
        const IORING_ENTER_SQ_WAKEUP = linux_rust_bindings::io_uring::IORING_ENTER_SQ_WAKEUP as u32;
        const IORING_ENTER_SQ_WAIT = linux_rust_bindings::io_uring::IORING_ENTER_SQ_WAIT as u32;
        const IORING_ENTER_EXT_ARG = linux_rust_bindings::io_uring::IORING_ENTER_EXT_ARG as u32;
        const IORING_ENTER_REGISTERED_RING = linux_rust_bindings::io_uring::IORING_ENTER_REGISTERED_RING as u32;
    }
}

transparent_bitflags! {
    pub struct IoUringFeatFlags: u32 {
        const IORING_FEAT_SINGLE_MMAP = linux_rust_bindings::io_uring::IORING_FEAT_SINGLE_MMAP as u32;
        const IORING_FEAT_NODROP = linux_rust_bindings::io_uring::IORING_FEAT_NODROP as u32;
        const IORING_FEAT_SUBMIT_STABLE = linux_rust_bindings::io_uring::IORING_FEAT_SUBMIT_STABLE as u32;
        const IORING_FEAT_RW_CUR_POS = linux_rust_bindings::io_uring::IORING_FEAT_RW_CUR_POS as u32;
        const IORING_FEAT_CUR_PERSONALITY = linux_rust_bindings::io_uring::IORING_FEAT_CUR_PERSONALITY as u32;
        const IORING_FEAT_FAST_POLL = linux_rust_bindings::io_uring::IORING_FEAT_FAST_POLL as u32;
        const IORING_FEAT_POLL_32BITS = linux_rust_bindings::io_uring::IORING_FEAT_POLL_32BITS as u32;
        const IORING_FEAT_SQPOLL_NONFIXED = linux_rust_bindings::io_uring::IORING_FEAT_SQPOLL_NONFIXED as u32;
        const IORING_FEAT_EXT_ARG = linux_rust_bindings::io_uring::IORING_FEAT_EXT_ARG as u32;
        const IORING_FEAT_NATIVE_WORKERS = linux_rust_bindings::io_uring::IORING_FEAT_NATIVE_WORKERS as u32;
        const IORING_FEAT_RSRC_TAGS = linux_rust_bindings::io_uring::IORING_FEAT_RSRC_TAGS as u32;
        const IORING_FEAT_CQE_SKIP = linux_rust_bindings::io_uring::IORING_FEAT_CQE_SKIP as u32;
        const IORING_FEAT_LINKED_FILE = linux_rust_bindings::io_uring::IORING_FEAT_LINKED_FILE as u32;
    }
}

#[repr(transparent)]
pub struct IoUringSubmissionQueueEntry(pub io_uring_sqe);

impl IoUringSubmissionQueueEntry {
    #[inline]
    #[must_use]
    pub fn new_readv(
        fd: Fd,
        file_offset: usize,
        buf_ptr: usize,
        num_buffers: u32,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: linux_rust_bindings::io_uring::io_uring_op_IORING_OP_READV as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                off: file_offset as u64,
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: buf_ptr as u64,
            },
            len: num_buffers as u32,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                // Todo: Accept `preadv` flags here https://man7.org/linux/man-pages//man2/preadv2.2.html
                rw_flags: 0,
            },
            user_data,
            __bindgen_anon_4: io_uring_sqe__bindgen_ty_4 { buf_index: 0 },
            personality: 0,
            __bindgen_anon_5: io_uring_sqe__bindgen_ty_5 { file_index: 0 },
            __bindgen_anon_6: io_uring_sqe__bindgen_ty_6 {
                __bindgen_anon_1: __BindgenUnionField::default(),
                cmd: __BindgenUnionField::default(),
                bindgen_union_field: [0; 2],
            },
        })
    }
    /// Creates a new entry that will execute an `openat` syscall.  
    /// # Safety
    /// It is up to the caller to make sure that the `path` reference lives until this
    /// entry is submitted or discarded.  
    /// [The docs](https://man7.org/linux/man-pages//man2/io_uring_enter2.2.html) doesn't
    /// say what will happen if it's freed before it's passed to the kernel.  
    /// It's likely to be an EINVAL but could be a `use-after-free`
    #[inline]
    #[must_use]
    pub unsafe fn new_openat(
        dir_fd: Option<Fd>,
        path: &UnixStr,
        open_flags: OpenFlags,
        mode: Mode,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: linux_rust_bindings::io_uring::io_uring_op_IORING_OP_OPENAT as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: dir_fd.unwrap_or(AT_FDCWD),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: path.0.as_ptr() as u64,
            },
            len: mode.bits(),
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                open_flags: open_flags.bits() as u32,
            },
            user_data,
            __bindgen_anon_4: io_uring_sqe__bindgen_ty_4 { buf_index: 0 },
            personality: 0,
            __bindgen_anon_5: io_uring_sqe__bindgen_ty_5 { file_index: 0 },
            __bindgen_anon_6: io_uring_sqe__bindgen_ty_6 {
                __bindgen_anon_1: __BindgenUnionField::default(),
                cmd: __BindgenUnionField::default(),
                bindgen_union_field: [0; 2],
            },
        })
    }
}

#[repr(transparent)]
#[derive(Debug)]
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
    pub(crate) flags: IoUringParamFlags,
    pub(crate) submission_queue: UringSubmissionQueue,
    pub(crate) completion_queue: UringCompletionQueue,
}

#[allow(dead_code)]
impl IoUring {
    #[inline]
    pub(crate) fn get_dropped(&self) -> u32 {
        unsafe {
            self.submission_queue
                .kernel_dropped
                .as_ref()
                .load(Ordering::Relaxed)
        }
    }

    pub(crate) fn get_next_sqe_slot(&mut self) -> Option<*mut IoUringSubmissionQueueEntry> {
        let next = self.submission_queue.tail as u32 + 1;
        let shift = u32::from(self.flags.contains(IoUringParamFlags::IORING_SETUP_SQE128));
        let head = if self.flags.contains(IoUringParamFlags::IORING_SETUP_SQPOLL) {
            self.submission_queue.acquire_khead()
        } else {
            self.submission_queue.get_khead_relaxed()
        };
        if next - head <= self.submission_queue.ring_entries {
            let index = (self.submission_queue.tail & self.submission_queue.ring_mask) << shift;
            let sqe = unsafe { self.submission_queue.entries.as_ptr().add(index as usize) };
            self.submission_queue.tail = next;
            Some(sqe)
        } else {
            None
        }
    }

    pub(crate) fn flush_submission_queue(&mut self) -> u32 {
        let tail = self.submission_queue.tail;
        if self.submission_queue.head != tail {
            self.submission_queue.head = tail;
            if self.flags.contains(IoUringParamFlags::IORING_SETUP_SQPOLL) {
                self.submission_queue.sync_ktail_release();
            } else {
                self.submission_queue.sync_ktail_relaxed();
            }
        }
        tail - self.submission_queue.get_khead_relaxed()
    }

    pub(crate) fn get_next_cqe(&mut self) -> Option<&IoUringCompletionQueueEntry> {
        let shift = u32::from(self.flags.contains(IoUringParamFlags::IORING_SETUP_CQE32));
        let tail = self.completion_queue.acquire_ktail();
        let head = self.completion_queue.acquire_khead();
        if tail <= head {
            return None;
        }
        let cqe = unsafe {
            self.completion_queue
                .entries
                .as_ptr()
                .add(((head & self.completion_queue.ring_mask) << shift) as usize)
        };
        self.completion_queue.advance(1);
        unsafe { cqe.as_ref() }
    }
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
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,
    pub(crate) entries: NonNull<IoUringSubmissionQueueEntry>,
}

impl UringSubmissionQueue {
    #[inline]
    pub(crate) fn get_khead_relaxed(&self) -> u32 {
        unsafe { (self.kernel_head.as_ref()).load(Ordering::Relaxed) }
    }

    #[inline]
    pub(crate) fn acquire_khead(&self) -> u32 {
        unsafe { (self.kernel_head.as_ref()).load(Ordering::Acquire) }
    }

    #[inline]
    pub(crate) fn sync_ktail_release(&self) {
        unsafe {
            self.kernel_tail
                .as_ref()
                .store(self.tail, Ordering::Release);
        }
    }

    #[inline]
    pub(crate) fn sync_ktail_relaxed(&self) {
        unsafe {
            self.kernel_tail
                .as_ref()
                .store(self.tail, Ordering::Relaxed);
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct UringCompletionQueue {
    pub(crate) ring_size: usize,
    pub(crate) ring_ptr: usize,
    pub(crate) kernel_head: NonNull<AtomicU32>,
    pub(crate) kernel_tail: NonNull<AtomicU32>,
    pub(crate) kernel_flags: Option<NonNull<AtomicU32>>,
    pub(crate) kernel_overflow: NonNull<AtomicU32>,
    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,
    pub(crate) entries: NonNull<IoUringCompletionQueueEntry>,
}

#[allow(dead_code)]
impl UringCompletionQueue {
    #[inline]
    pub(crate) fn acquire_ktail(&self) -> u32 {
        unsafe { self.kernel_tail.as_ref().load(Ordering::Acquire) }
    }

    #[inline]
    pub(crate) fn acquire_khead(&self) -> u32 {
        unsafe { self.kernel_head.as_ref().load(Ordering::Acquire) }
    }

    #[inline]
    pub(crate) fn get_khead_relaxed(&self) -> u32 {
        unsafe { self.kernel_head.as_ref().load(Ordering::Relaxed) }
    }

    #[inline]
    pub(crate) fn advance(&mut self, num: u32) {
        unsafe { self.kernel_head.as_ref().fetch_add(num, Ordering::Release) };
    }
}
