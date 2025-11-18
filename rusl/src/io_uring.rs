use core::num::NonZeroUsize;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};

use linux_rust_bindings::io_uring::{
    IORING_OFF_CQ_RING, IORING_OFF_SQES, IORING_OFF_SQ_RING, IORING_REGISTER_BUFFERS,
    IORING_REGISTER_FILES,
};
use sc::syscall;

use crate::platform::{
    Fd, IoSliceMut, IoUring, IoUringCompletionQueueEntry, IoUringEnterFlags, IoUringFeatFlags,
    IoUringParamFlags, IoUringParams, IoUringSubmissionQueueEntry, MapAdditionalFlags,
    MapRequiredFlag, MemoryProtection, UringCompletionQueue, UringSubmissionQueue,
};
use crate::unistd::mmap;
use crate::{Error, Result};

#[cfg(test)]
mod test;

/// Creates an `IoUring` instance with shared memory between user and kernel space.\
/// `entries` are the number of available slots in the submission queue,\
/// 'flags' are passed to `io_uring_setup`.\
/// See the [linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_setup.2.html)  
/// # Errors
/// See above
#[expect(clippy::too_many_lines)]
pub fn setup_io_uring(
    entries: u32,
    flags: IoUringParamFlags,
    sq_thread_cpu: u32,
    sq_thread_idle: u32,
) -> Result<IoUring> {
    let mut params = IoUringParams::new(flags, sq_thread_cpu, sq_thread_idle);
    let fd = io_uring_setup(entries, &mut params)?;
    let mut cq_size = core::mem::size_of::<IoUringCompletionQueueEntry>();
    if flags.contains(IoUringParamFlags::IORING_SETUP_CQE32) {
        cq_size += core::mem::size_of::<IoUringCompletionQueueEntry>();
    }
    let mut sq_ring_sz =
        params.0.sq_off.array as usize + params.0.sq_entries as usize * core::mem::size_of::<u32>();
    let mut cq_ring_sz = (params.0.cq_off.cqes) as usize + params.0.cq_entries as usize * cq_size;
    if params.0.features & IoUringFeatFlags::IORING_FEAT_SINGLE_MMAP.bits() != 0 {
        if cq_ring_sz > sq_ring_sz {
            sq_ring_sz = cq_ring_sz;
        } else {
            cq_ring_sz = sq_ring_sz;
        }
    }

    unsafe {
        let sq_ring_ptr = crate::unistd::mmap(
            None,
            // Safety: The kernel rejects 0 entries as `EINVAL` and the size isn't 0
            NonZeroUsize::new_unchecked(sq_ring_sz),
            MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
            MapRequiredFlag::MapShared,
            MapAdditionalFlags::MAP_POPULATE,
            Some(fd),
            i64::from(IORING_OFF_SQ_RING),
        )?;
        let cq_ring_ptr =
            if params.0.features & IoUringFeatFlags::IORING_FEAT_SINGLE_MMAP.bits() == 0 {
                // cq offset from https://kernel.dk/io_uring.pdf
                mmap(
                    None,
                    // Safety: The kernel rejects 0 entries as `EINVAL` and the size isn't 0
                    NonZeroUsize::new_unchecked(cq_ring_sz),
                    MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
                    MapRequiredFlag::MapShared,
                    MapAdditionalFlags::MAP_POPULATE,
                    Some(fd),
                    i64::from(IORING_OFF_CQ_RING),
                )?
            } else {
                sq_ring_ptr
            };
        let sq_khead = into_non_null(sq_ring_ptr, params.0.sq_off.head as usize)?;
        let sq_ktail = into_non_null(sq_ring_ptr, params.0.sq_off.tail as usize)?;
        let sq_kflags = into_non_null(sq_ring_ptr, params.0.sq_off.flags as usize)?;
        let sq_kdropped = into_non_null(sq_ring_ptr, params.0.sq_off.dropped as usize)?;
        let sq_array = into_non_null(sq_ring_ptr, params.0.sq_off.array as usize)?;
        let mut sqe_size = core::mem::size_of::<IoUringSubmissionQueueEntry>();
        // TODO: Use constant, 10 is 128bit SQE
        if params.0.flags & 1 << 10 != 0 {
            sqe_size += 64;
        }
        // sqes offset from https://kernel.dk/io_uring.pdf
        let sqes = mmap(
            None,
            // Safety: The kernel rejects 0 entries as `EINVAL` and the size isn't 0
            NonZeroUsize::new_unchecked(sqe_size * params.0.sq_entries as usize),
            MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
            MapRequiredFlag::MapShared,
            MapAdditionalFlags::MAP_POPULATE,
            Some(fd),
            i64::from(IORING_OFF_SQES),
        )?;
        let sqes = NonNull::new_unchecked(sqes as *mut IoUringSubmissionQueueEntry);
        let cq_khead = into_non_null(cq_ring_ptr, params.0.cq_off.head as usize)?;
        let cq_ktail = into_non_null(cq_ring_ptr, params.0.cq_off.tail as usize)?;
        let cq_koverflow = into_non_null(cq_ring_ptr, params.0.cq_off.overflow as usize)?;
        let cq_cqes = into_non_null(cq_ring_ptr, params.0.cq_off.cqes as usize)?.cast();
        let cq_kflags = if params.0.cq_off.flags == 0 {
            None
        } else {
            NonNull::new((cq_ring_ptr + params.0.cq_off.flags as usize) as *mut AtomicU32)
        };
        let sq_ring_mask = value_at_offset(sq_ring_ptr, params.0.sq_off.ring_mask as usize)?;
        let sq_ring_entries = value_at_offset(sq_ring_ptr, params.0.sq_off.ring_entries as usize)?;
        let cq_ring_mask = value_at_offset(cq_ring_ptr, params.0.cq_off.ring_mask as usize)?;
        let cq_ring_entries = value_at_offset(cq_ring_ptr, params.0.cq_off.ring_entries as usize)?;
        // Map SQ-slots to SQEs, like in liburing
        for index in 0..sq_ring_entries {
            (*sq_array.as_ptr().add(index as usize)).store(index, Ordering::Release);
        }
        // Safety: All pointers are guaranteed to not be a null-pointer,
        // we get them from a successful `mmap`
        Ok(IoUring {
            fd,
            flags,
            submission_queue: UringSubmissionQueue {
                ring_size: sq_ring_sz,
                ring_ptr: sq_ring_ptr,
                kernel_head: sq_khead,
                kernel_tail: sq_ktail,
                kernel_flags: sq_kflags,
                kernel_dropped: sq_kdropped,
                kernel_array: sq_array,
                head: 0,
                tail: 0,
                ring_mask: sq_ring_mask,
                ring_entries: sq_ring_entries,
                entries: sqes,
            },
            completion_queue: UringCompletionQueue {
                ring_size: cq_ring_sz,
                ring_ptr: cq_ring_ptr,
                kernel_head: cq_khead,
                kernel_tail: cq_ktail,
                kernel_flags: cq_kflags,
                kernel_overflow: cq_koverflow,
                ring_mask: cq_ring_mask,
                ring_entries: cq_ring_entries,
                entries: cq_cqes,
            },
        })
    }
}

#[inline]
fn value_at_offset(src_ptr: usize, bit_offset: usize) -> Result<u32> {
    let ptr = NonNull::new((src_ptr + bit_offset) as *mut AtomicU32)
        .ok_or(Error::no_code("Got a nullptr from io uring setup"))?;
    unsafe { Ok(ptr.as_ref().load(Ordering::Relaxed)) }
}

#[inline]
unsafe fn into_non_null(src_addr: usize, bit_offset: usize) -> Result<NonNull<AtomicU32>> {
    NonNull::new((src_addr + bit_offset) as *mut AtomicU32)
        .ok_or(Error::no_code("Got a nullptr from io uring setup"))
}

/// Sets up a new `io_uring` instance fitting `entries` amount of entries
/// returning its `fd`.\
/// See [Linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_setup.2.html)
/// # Errors
/// See above  
#[inline]
pub fn io_uring_setup(entries: u32, io_uring_params: &mut IoUringParams) -> Result<Fd> {
    let res = unsafe {
        syscall!(
            IO_URING_SETUP,
            entries,
            core::ptr::from_mut::<IoUringParams>(io_uring_params)
        )
    };
    Fd::coerce_from_register(res, "`IO_URING_SETUP` syscall failed")
}

/// Register files on an `io_uring` instance.\
/// See [Linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_register.2.html)  
/// # Errors
/// See above  
#[inline]
pub fn io_uring_register_files(uring_fd: Fd, fds: &[Fd]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd.0,
            IORING_REGISTER_FILES,
            fds.as_ptr(),
            fds.len()
        )
    };
    bail_on_below_zero!(res, "`IO_URING_REGISTER` Syscall failed registering files");
    Ok(())
}

/// Register io slices on an `io_uring` instance.
/// See [Linux documentation for details]()
/// # Errors
/// See above
#[inline]
pub fn io_uring_register_io_slices(uring_fd: Fd, buffers: &[IoSliceMut]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd.0,
            IORING_REGISTER_BUFFERS,
            buffers.as_ptr(),
            buffers.len()
        )
    };
    bail_on_below_zero!(
        res,
        "`IO_URING_REGISTER` Syscall failed registering io slices"
    );
    Ok(())
}

/// Register the contained io slices, the parent buffer does not need to live longer than until
/// the completion of this syscall.
/// See [Linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_register.2.html)
/// # Errors
/// See above
/// # Safety
/// The buffers must not be used by `uring` fixed-buffer calls after the buffers are deallocated.  
#[inline]
pub unsafe fn io_uring_register_buffers(uring_fd: Fd, buffer: &[IoSliceMut]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd.0,
            IORING_REGISTER_BUFFERS,
            buffer.as_ptr(),
            buffer.len()
        )
    };
    bail_on_below_zero!(res, "`IO_URING_REGISTER` Syscall failed registering buffer");
    Ok(())
}

/// Initiate and complete io using the shared submission and completion queue of the
/// already setup `io_uring` at `uring_fd`.
/// See [linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_enter.2.html)
/// # Errors
/// See above
#[inline]
pub fn io_uring_enter(
    uring_fd: Fd,
    to_submit: u32,
    min_complete: u32,
    flags: IoUringEnterFlags,
) -> Result<usize> {
    let res = unsafe {
        syscall!(
            IO_URING_ENTER,
            uring_fd.0,
            to_submit,
            min_complete,
            flags.bits(),
            0
        )
    };
    bail_on_below_zero!(res, "`IO_URING_ENTER` syscall failed");
    Ok(res)
}
