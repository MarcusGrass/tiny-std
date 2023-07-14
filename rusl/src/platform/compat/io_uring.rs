use core::fmt::{Debug, Formatter};
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};

use linux_rust_bindings::io_uring::{
    __BindgenUnionField, io_cqring_offsets, io_sqring_offsets, io_uring_cqe, io_uring_params,
    io_uring_sqe, io_uring_sqe__bindgen_ty_1, io_uring_sqe__bindgen_ty_2,
    io_uring_sqe__bindgen_ty_3, io_uring_sqe__bindgen_ty_4, io_uring_sqe__bindgen_ty_5,
    io_uring_sqe__bindgen_ty_6, IORING_SETUP_SQE128, IORING_SQ_NEED_WAKEUP, IORING_TIMEOUT_ABS,
};

use crate::platform::{
    comptime_i32_to_u32, comptime_u32_to_u8, AddressFamily, Fd, Mode, OpenFlags, RenameFlags,
    SocketArg, SocketType, Statx, StatxFlags, StatxMask, TimeSpec, AT_FDCWD, AT_REMOVEDIR,
};
use crate::string::unix_str::UnixStr;
use crate::unistd::munmap;

transparent_bitflags! {
    pub struct IoUringParamFlags: u32 {
        const DEFAULT = 0;
        const IORING_SETUP_IOPOLL = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_IOPOLL);
        const IORING_SETUP_SQPOLL = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_SQPOLL);
        const IORING_SETUP_SQ_AFF = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_SQ_AFF);
        const IORING_SETUP_CQSIZE = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_CQSIZE);
        const IORING_SETUP_CLAMP = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_CLAMP);
        const IORING_SETUP_ATTACH_WQ = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_ATTACH_WQ);
        const IORING_SETUP_R_DISABLED = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_R_DISABLED);
        const IORING_SETUP_SUBMIT_ALL = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_SUBMIT_ALL);
        const IORING_SETUP_COOP_TASKRUN = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_COOP_TASKRUN);
        const IORING_SETUP_TASKRUN_FLAG = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_TASKRUN_FLAG);
        const IORING_SETUP_SQE128 = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_SQE128);
        const IORING_SETUP_CQE32 = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_CQE32);
        const IORING_SETUP_SINGLE_ISSUER = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_SINGLE_ISSUER);
        const IORING_SETUP_DEFER_TASKRUN = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_SETUP_DEFER_TASKRUN);
    }
}

transparent_bitflags! {
    pub struct IoUringSQEFlags: u8 {
        const DEFAULT = 0;
        const IOSQE_FIXED_FILE = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_FIXED_FILE_BIT);
        const IOSQE_IO_DRAIN = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_IO_DRAIN_BIT);
        const IOSQE_IO_LINK = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_IO_LINK_BIT);
        const IOSQE_IO_HARDLINK = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_IO_HARDLINK_BIT);
        const IOSQE_ASYNC = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_ASYNC_BIT);
        const IOSQE_BUFFER_SELECT = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_BUFFER_SELECT_BIT);
        const IOSQE_CQE_SKIP_SUCCESS = 1 << comptime_u32_to_u8(linux_rust_bindings::io_uring::IOSQE_CQE_SKIP_SUCCESS_BIT);
    }
}

transparent_bitflags! {
    pub struct IoUringEnterFlags: u32 {
        const DEFAULT = 0;
        const IORING_ENTER_GETEVENTS = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_ENTER_GETEVENTS);
        const IORING_ENTER_SQ_WAKEUP = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_ENTER_SQ_WAKEUP);
        const IORING_ENTER_SQ_WAIT = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_ENTER_SQ_WAIT);
        const IORING_ENTER_EXT_ARG = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_ENTER_EXT_ARG);
        const IORING_ENTER_REGISTERED_RING = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_ENTER_REGISTERED_RING);
    }
}

transparent_bitflags! {
    pub struct IoUringFeatFlags: u32 {
        const DEFAULT = 0;
        const IORING_FEAT_SINGLE_MMAP = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_SINGLE_MMAP);
        const IORING_FEAT_NODROP = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_NODROP);
        const IORING_FEAT_SUBMIT_STABLE = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_SUBMIT_STABLE);
        const IORING_FEAT_RW_CUR_POS = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_RW_CUR_POS);
        const IORING_FEAT_CUR_PERSONALITY = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_CUR_PERSONALITY);
        const IORING_FEAT_FAST_POLL = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_FAST_POLL);
        const IORING_FEAT_POLL_32BITS = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_POLL_32BITS);
        const IORING_FEAT_SQPOLL_NONFIXED = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_SQPOLL_NONFIXED);
        const IORING_FEAT_EXT_ARG = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_EXT_ARG);
        const IORING_FEAT_NATIVE_WORKERS = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_NATIVE_WORKERS);
        const IORING_FEAT_RSRC_TAGS = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_RSRC_TAGS);
        const IORING_FEAT_CQE_SKIP = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_CQE_SKIP);
        const IORING_FEAT_LINKED_FILE = comptime_i32_to_u32(linux_rust_bindings::io_uring::IORING_FEAT_LINKED_FILE);
    }
}

#[repr(u8)]
pub enum IoUringOp {
    Nop = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_NOP),
    Readv = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_READV),
    Writev = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_WRITEV),
    Fsync = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_FSYNC),
    ReadFixed = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_READ_FIXED),
    WriteFixed =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_WRITE_FIXED),
    PollFdd = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_POLL_ADD),
    PollRemove =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_POLL_REMOVE),
    SyncFileRange =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SYNC_FILE_RANGE),
    Sendmsg = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SENDMSG),
    Recvmsg = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_RECVMSG),
    Timeout = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_TIMEOUT),
    TimeoutRemove =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_TIMEOUT_REMOVE),
    Accept = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_ACCEPT),
    AsyncCancel =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_ASYNC_CANCEL),
    LinkTimeout =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_LINK_TIMEOUT),
    Connect = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_CONNECT),
    Fallocate = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_FALLOCATE),
    Openat = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_OPENAT),
    Close = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_CLOSE),
    FilesUpdate =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_FILES_UPDATE),
    Statx = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_STATX),
    Read = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_READ),
    Write = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_WRITE),
    Fadvise = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_FADVISE),
    Madvise = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_MADVISE),
    Send = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SEND),
    Recv = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_RECV),
    Openat2 = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_OPENAT2),
    EpollCtl = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_EPOLL_CTL),
    Splice = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SPLICE),
    ProvideBuffers =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_PROVIDE_BUFFERS),
    RemoveBuffers =
        comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_REMOVE_BUFFERS),
    Tee = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_TEE),
    Shutdown = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SHUTDOWN),
    Renameat = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_RENAMEAT),
    Unlinkat = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_UNLINKAT),
    Mkdirat = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_MKDIRAT),
    Symlinkat = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SYMLINKAT),
    Linkat = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_LINKAT),
    MsgRing = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_MSG_RING),
    Fsetxattr = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_FSETXATTR),
    Setxattr = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SETXATTR),
    Fgetxattr = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_FGETXATTR),
    Getxattr = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_GETXATTR),
    Socket = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SOCKET),
    UringCmd = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_URING_CMD),
    SendZc = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SEND_ZC),
    SendmsgZc = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_SENDMSG_ZC),
    Last = comptime_u32_to_u8(linux_rust_bindings::io_uring::io_uring_op_IORING_OP_LAST),
}

#[repr(transparent)]
pub struct IoUringSubmissionQueueEntry(pub io_uring_sqe);

impl Debug for IoUringSubmissionQueueEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IoUringSubmissionQueueEntry")
            .field("opcode", &self.0.opcode)
            .field("flags", &self.0.flags)
            .field("user_data", &self.0.user_data)
            .finish()
    }
}

impl IoUringSubmissionQueueEntry {
    /// Read vectored into a buffer.
    /// # Safety
    /// The underlying buffer needs to live at least until this `sqe` is submitted to the kernel.
    #[inline]
    #[must_use]
    pub unsafe fn new_readv(
        fd: Fd,
        buf_ptr: usize,
        num_buffers: u32,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Readv as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: fd.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: buf_ptr as u64,
            },
            len: num_buffers,
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

    /// Read vectored into a pre-registered buffer, buffers are registered with `io_uring_register`.   
    /// # Safety
    /// The underlying buffer needs to live at least until this `sqe` is completed.  
    #[inline]
    #[must_use]
    pub unsafe fn new_readv_fixed(
        fd: Fd,
        buf_ind: u16,
        start_read_into_addr: u64,
        read_exact: u32,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::ReadFixed as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: fd.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: start_read_into_addr,
            },
            len: read_exact,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                // Todo: Accept `preadv` flags here https://man7.org/linux/man-pages//man2/preadv2.2.html
                rw_flags: 0,
            },
            user_data,
            __bindgen_anon_4: io_uring_sqe__bindgen_ty_4 { buf_index: buf_ind },
            personality: 0,
            __bindgen_anon_5: io_uring_sqe__bindgen_ty_5 { file_index: 0 },
            __bindgen_anon_6: io_uring_sqe__bindgen_ty_6 {
                __bindgen_anon_1: __BindgenUnionField::default(),
                cmd: __BindgenUnionField::default(),
                bindgen_union_field: [0; 2],
            },
        })
    }

    /// Write vectored from a buffer.  
    /// # Safety
    /// The underlying buffer needs to live at least until this `sqe` is submitted to the kernel
    #[inline]
    #[must_use]
    pub unsafe fn new_writev(
        fd: Fd,
        buf_ptr: usize,
        num_buffers: u32,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Writev as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: fd.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: buf_ptr as u64,
            },
            len: num_buffers,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                // Todo: Accept `pwritev` flags here https://man7.org/linux/man-pages//man2/preadv2.2.html
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

    /// Write vectored from a fixed previously registered buffer (`io_uring_register`).  
    /// # Safety
    /// The underlying buffer needs to live at least until this `sqe` is completed
    #[inline]
    #[must_use]
    pub unsafe fn new_writev_fixed(
        fd: Fd,
        buf_ind: u16,
        start_read_into_addr: u64,
        write_exact: u32,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::WriteFixed as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: fd.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: start_read_into_addr,
            },
            len: write_exact,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                // Todo: Accept `pwritev` flags here https://man7.org/linux/man-pages//man2/preadv2.2.html
                rw_flags: 0,
            },
            user_data,
            __bindgen_anon_4: io_uring_sqe__bindgen_ty_4 { buf_index: buf_ind },
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
            opcode: IoUringOp::Openat as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: dir_fd.map_or(AT_FDCWD, |fd| fd.0),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: path.0.as_ptr() as u64,
            },
            len: mode.bits(),
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                open_flags: open_flags.bits().into_u32(),
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

    /// Creates a new entry that will execute an equivalent to a `close` syscall
    #[inline]
    #[must_use]
    pub fn new_close(fd: Fd, user_data: u64, sqe_flags: IoUringSQEFlags) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Close as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: fd.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 { addr: 0 },
            len: 0,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 { open_flags: 0 },
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

    /// Creates a new entry that will execute an equivalent to a `statx` syscall  
    /// # Safety
    /// The references `path` and the memory backing `statx_ptr`
    /// needs to live until this entry is submitted to the kernel.
    #[inline]
    #[must_use]
    pub unsafe fn new_statx(
        dir_fd: Option<Fd>,
        path: &UnixStr,
        flags: StatxFlags,
        mask: StatxMask,
        statx_ptr: *mut Statx,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Statx as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: dir_fd.map_or(AT_FDCWD, |fd| fd.0),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                off: statx_ptr as u64,
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: path.0.as_ptr() as u64,
            },
            len: mask.bits(),
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                statx_flags: flags.bits().into_u32(),
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

    /// Creates a new entry that will execute an equivalent to an `unlinkat` syscall.  
    /// # Safety
    /// The reference to `path` needs to live until this entry is submitted to the kernel.  
    #[inline]
    #[must_use]
    pub unsafe fn new_unlink_at(
        dir_fd: Option<Fd>,
        path: &UnixStr,
        rmdir: bool,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Unlinkat as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: dir_fd.map_or(AT_FDCWD, |fd| fd.0),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: path.0.as_ptr() as u64,
            },
            len: 0,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                unlink_flags: if rmdir { AT_REMOVEDIR.into_u32() } else { 0 },
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

    /// Creates a new entry that will execute an equivalent to an `renameat2` syscall.  
    /// # Safety
    /// The references to `old_path` and `new_path` needs to live until this entry is submitted to the kernel.  
    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub unsafe fn new_rename_at(
        old_dir_fd: Option<Fd>,
        new_dir_fd: Option<Fd>,
        old_path: &UnixStr,
        new_path: &UnixStr,
        flags: RenameFlags,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Renameat as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: old_dir_fd.map_or(AT_FDCWD, |fd| fd.0),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                addr2: new_path.0.as_ptr() as u64,
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: old_path.0.as_ptr() as u64,
            },
            len: new_dir_fd.map_or(AT_FDCWD as u32, Fd::into_u32),
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                rename_flags: flags.bits(),
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

    /// Creates a new entry that will execute an equivalent to an `mkdirat2` syscall.  
    /// # Safety
    /// The references to `old_path` and `new_path` needs to live until this entry is submitted to the kernel.  
    #[inline]
    #[must_use]
    pub unsafe fn new_mkdirat(
        dir_fd: Option<Fd>,
        path: &UnixStr,
        mode: Mode,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Mkdirat as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: dir_fd.map_or(AT_FDCWD, |fd| fd.0),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 { off: 0 },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: path.0.as_ptr() as u64,
            },
            len: mode.bits(),
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 { rename_flags: 0 },
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

    /// Creates a new socket. Will execute an equivalent to an `socket2` syscall.  
    #[inline]
    #[must_use]
    pub fn new_socket(
        domain: AddressFamily,
        socket_type: SocketType,
        protocol: u32,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Socket as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: i32::from(domain.bits()),
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                off: socket_type.bits().into_u64(),
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 { addr: 0 },
            len: protocol,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 { rw_flags: 0 },
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
    /// Creates a new socket. Will execute an equivalent to an `connect` syscall.  
    /// # Safety
    /// `sockaddr` needs to live until this entry is passed to the kernel
    #[inline]
    #[must_use]
    pub unsafe fn new_connect(
        socket: Fd,
        sockaddr: &SocketArg,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Connect as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: socket.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                off: core::ptr::addr_of!(sockaddr.addr_len) as u64,
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: core::ptr::addr_of!(sockaddr.addr) as u64,
            },
            len: 0,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 { rw_flags: 0 },
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

    /// Creates a new socket. Will execute an equivalent to an `accept4` syscall.  
    /// # Safety
    /// `sockaddr` needs to live until this entry is passed to the kernel
    #[inline]
    #[must_use]
    pub unsafe fn new_accept(
        socket: Fd,
        sockaddr: &SocketArg,
        sock_type: SocketType,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Accept as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: socket.0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                addr2: core::ptr::addr_of!(sockaddr.addr_len) as u64,
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: core::ptr::addr_of!(sockaddr.addr) as u64,
            },
            len: 0,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                accept_flags: sock_type.bits().into_u32(),
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

    /// Enters a timeout entry, will produce a cqe with result `-ETIME` on elapse or 0
    /// if `await_completions` is specified and that number of cqes have completed during
    /// the timeout duration.  
    /// # Safety
    /// `ts` needs to live until this entry is passed to the kernel
    #[inline]
    #[must_use]
    pub unsafe fn new_timeout(
        ts: &TimeSpec,
        relative: bool,
        await_completions: Option<u64>,
        user_data: u64,
        sqe_flags: IoUringSQEFlags,
    ) -> Self {
        Self(io_uring_sqe {
            opcode: IoUringOp::Timeout as u8,
            flags: sqe_flags.bits(),
            ioprio: 0,
            fd: 0,
            __bindgen_anon_1: io_uring_sqe__bindgen_ty_1 {
                off: await_completions.unwrap_or_default(),
            },
            __bindgen_anon_2: io_uring_sqe__bindgen_ty_2 {
                addr: ts as *const TimeSpec as u64,
            },
            len: 1,
            __bindgen_anon_3: io_uring_sqe__bindgen_ty_3 {
                timeout_flags: if relative {
                    0
                } else {
                    IORING_TIMEOUT_ABS as u32
                },
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
    pub const fn new(flags: IoUringParamFlags, sq_thread_cpu: u32, sq_thread_idle: u32) -> Self {
        Self(io_uring_params {
            sq_entries: 0,
            cq_entries: 0,
            flags: flags.bits(),
            sq_thread_cpu,
            sq_thread_idle,
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
                user_addr: 0,
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
                user_addr: 0,
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

    #[inline]
    #[must_use]
    pub fn needs_wakeup(&self) -> bool {
        unsafe {
            self.submission_queue
                .kernel_flags
                .as_ref()
                .load(Ordering::Acquire)
                & IORING_SQ_NEED_WAKEUP as u32
                != 0
        }
    }

    pub fn get_next_sqe_slot(&mut self) -> Option<*mut IoUringSubmissionQueueEntry> {
        let next = self.submission_queue.tail + 1;
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

    pub fn flush_submission_queue(&mut self) -> u32 {
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

    pub fn get_next_cqe(&mut self) -> Option<&IoUringCompletionQueueEntry> {
        let shift = u32::from(self.flags.contains(IoUringParamFlags::IORING_SETUP_CQE32));
        let tail = self.completion_queue.acquire_ktail();
        let head = self.completion_queue.acquire_khead();
        if tail <= head {
            return None;
        }
        let ind = ((head & self.completion_queue.ring_mask) << shift) as usize;
        let cqe = unsafe { self.completion_queue.entries.as_ptr().add(ind) };
        self.completion_queue.advance(1);
        unsafe { cqe.as_ref() }
    }
}

impl Drop for IoUring {
    #[allow(clippy::let_underscore_untyped)]
    fn drop(&mut self) {
        let mut sqe_size = core::mem::size_of::<IoUringSubmissionQueueEntry>();
        if self.flags.bits() & IORING_SETUP_SQE128 as u32 != 0 {
            sqe_size += 64;
        }
        unsafe {
            let _ = munmap(
                self.submission_queue.entries.as_ptr() as usize,
                NonZeroUsize::new(self.submission_queue.ring_entries as usize * sqe_size).unwrap(),
            );
            let _ = munmap(
                self.submission_queue.ring_ptr,
                NonZeroUsize::new(self.submission_queue.ring_size).unwrap(),
            );
            let _ = munmap(
                self.completion_queue.ring_ptr,
                NonZeroUsize::new(self.completion_queue.ring_size).unwrap(),
            );
        }
        let _ = crate::unistd::close(self.fd);
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
