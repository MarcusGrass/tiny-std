use core::num::NonZeroUsize;
use core::ptr::NonNull;

use linux_rust_bindings::io_uring::{IORING_REGISTER_BUFFERS, IORING_REGISTER_FILES};
use sc::syscall;

use crate::platform::{
    Fd, IoSliceMut, IoUring, IoUringCompletionQueueEntry, IoUringParamFlags, IoUringParams,
    IoUringSubmissionQueueEntry, MapAdditionalFlags, MapRequiredFlag, MemoryProtection,
    UringCompletionQueue, UringSubmissionQueue,
};
use crate::unistd::mmap;
use crate::Result;

/// Creates an `IoUring` instance with shared memory between user and kernel space.  
/// `entries` are the number of available slots in the submission queue,  
/// 'flags' are passed to `io_uring_setup`.  
/// See the [linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_setup.2.html)  
/// # Errors
/// See above
pub fn setup_io_uring(entries: u32, flags: IoUringParamFlags) -> Result<IoUring> {
    let mut params = IoUringParams::new(flags);
    let fd = io_uring_setup(entries, &mut params)?;
    let cq_size = core::mem::size_of::<IoUringCompletionQueueEntry>();
    let sq_ring_sz =
        params.0.sq_off.array as usize + params.0.sq_entries as usize * core::mem::size_of::<u32>();
    let mut cq_ring_sz = (params.0.cq_off.cqes + params.0.cq_entries) as usize * cq_size;
    if params.0.features & 1 != 0 {
        cq_ring_sz = sq_ring_sz;
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
            0,
        )?;
        // TODO: Use constant, 1 is SINGLE_MMAP
        let cq_ring_ptr = if params.0.features & 1 == 0 {
            // cq offset from https://kernel.dk/io_uring.pdf
            mmap(
                None,
                // Safety: The kernel rejects 0 entries as `EINVAL` and the size isn't 0
                NonZeroUsize::new_unchecked(cq_ring_sz),
                MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
                MapRequiredFlag::MapShared,
                MapAdditionalFlags::MAP_POPULATE,
                Some(fd),
                0x8000000,
            )?
        } else {
            sq_ring_ptr
        };
        let sq_khead = sq_ring_ptr + params.0.sq_off.head as usize;
        let sq_ktail = sq_ring_ptr + params.0.sq_off.tail as usize;
        let sq_kflags = sq_ring_ptr + params.0.sq_off.flags as usize;
        let sq_kdropped = sq_ring_ptr + params.0.sq_off.dropped as usize;
        let sq_array = sq_ring_ptr + params.0.sq_off.array as usize;
        let mut size = core::mem::size_of::<IoUringSubmissionQueueEntry>();
        // TODO: Use constant, 10 is 128bit SQE
        if params.0.flags & 1 << 10 != 0 {
            size += 64;
        }
        // sqes offset from https://kernel.dk/io_uring.pdf
        let sqes = mmap(
            None,
            // Safety: The kernel rejects 0 entries as `EINVAL` and the size isn't 0
            NonZeroUsize::new_unchecked(size * params.0.sq_entries as usize),
            MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
            MapRequiredFlag::MapShared,
            MapAdditionalFlags::MAP_POPULATE,
            Some(fd),
            0x10000000,
        )?;
        let cq_khead = cq_ring_ptr + params.0.cq_off.head as usize;
        let cq_ktail = cq_ring_ptr + params.0.cq_off.tail as usize;
        let cq_koverflow = cq_ring_ptr + params.0.cq_off.overflow as usize;
        let cq_cqes = cq_ring_ptr + params.0.cq_off.cqes as usize;
        let cq_kflags = if params.0.cq_off.flags == 0 {
            0
        } else {
            cq_ring_ptr + params.0.cq_off.flags as usize
        };
        let sq_ring_mask = *((sq_ring_ptr + params.0.sq_off.ring_mask as usize) as *const u32);
        let sq_ring_entries =
            *((sq_ring_ptr + params.0.sq_off.ring_entries as usize) as *const u32);
        let cq_ring_mask = *((cq_ring_ptr + params.0.cq_off.ring_mask as usize) as *const u32);
        let cq_ring_entries =
            *((cq_ring_ptr + params.0.cq_off.ring_entries as usize) as *const u32);

        // Safety: All pointers are guaranteed to not be a null-pointer,
        // we get them from a successful `mmap`
        Ok(IoUring {
            fd,
            sq: UringSubmissionQueue {
                ring_size: sq_ring_sz,
                ring_ptr: sq_ring_ptr,
                kernel_head: NonNull::new_unchecked(sq_khead as _),
                kernel_tail: NonNull::new_unchecked(sq_ktail as _),
                kernel_flags: NonNull::new_unchecked(sq_kflags as _),
                kernel_dropped: NonNull::new_unchecked(sq_kdropped as _),
                kernel_array: NonNull::new_unchecked(sq_array as _),
                head: 0,
                tail: 0,
                ring_mask: sq_ring_mask,
                ring_entries: sq_ring_entries,
                entries: NonNull::new_unchecked(sqes as _),
            },
            cq: UringCompletionQueue {
                ring_size: cq_ring_sz,
                ring_ptr: cq_ring_ptr,
                kernel_head: NonNull::new_unchecked(cq_khead as _),
                kernel_tail: NonNull::new_unchecked(cq_ktail as _),
                kernel_flags: NonNull::new_unchecked(cq_kflags as _),
                kernel_overflow: NonNull::new_unchecked(cq_koverflow as _),
                ring_mask: cq_ring_mask,
                ring_entries: cq_ring_entries,
                entries: NonNull::new_unchecked(cq_cqes as _),
            },
        })
    }
}

/// Sets up a new `io_uring` instance fitting `entries` amount of entries
/// returning its `fd`.  
/// See [Linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_setup.2.html)
/// # Errors
/// See above  
#[inline]
pub fn io_uring_setup(entries: u32, io_uring_params: &mut IoUringParams) -> Result<Fd> {
    let res = unsafe {
        syscall!(
            IO_URING_SETUP,
            entries,
            io_uring_params as *mut IoUringParams
        )
    };
    bail_on_below_zero!(res, "`IO_URING_SETUP` syscall failed");
    Ok(res as Fd)
}

/// Register files on an `io_uring` instance.  
/// See [Linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_register.2.html)  
/// # Errors
/// See above  
#[inline]
pub fn io_uring_register_files(uring_fd: Fd, fds: &[Fd]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd,
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
            uring_fd,
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

/// Register a fixed buffer on an `io_uring` instance.  
/// See [Linux documentation for details](https://man7.org/linux/man-pages//man2/io_uring_register.2.html)
/// # Errors
/// See above
#[inline]
pub fn io_uring_register_buf(uring_fd: Fd, buffer: &[u8]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd,
            IORING_REGISTER_BUFFERS,
            buffer.as_ptr(),
            1
        )
    };
    bail_on_below_zero!(res, "`IO_URING_REGISTER` Syscall failed registering buffer");
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::io_uring::{
        io_uring_register_buf, io_uring_register_files, io_uring_register_io_slices,
        io_uring_setup, setup_io_uring,
    };
    use crate::platform::{Fd, IoSliceMut, IoUringParamFlags, IoUringParams, OpenFlags};
    use crate::unistd::open;

    #[test]
    fn uring_setup() {
        let _ = setup_io_poll_uring();
    }

    fn setup_io_poll_uring() -> Option<Fd> {
        let mut params = IoUringParams::new(IoUringParamFlags::IORING_SETUP_IOPOLL);
        let uring_fd = match io_uring_setup(1, &mut params) {
            Ok(uring_fd) => {
                assert_ne!(0, uring_fd);
                uring_fd
            }
            #[allow(unused_variables)]
            Err(e) => {
                #[cfg(target_arch = "aarch64")]
                if e.code.unwrap() != crate::error::Errno::ENOSYS {
                    panic!("{}", e);
                }
                return None;
            }
        };
        Some(uring_fd)
    }

    #[test]
    fn uring_register_files() {
        let Some(uring_fd) = setup_io_poll_uring() else {
            return;
        };
        let open_fd = open("test-files/can_open.txt\0", OpenFlags::O_RDWR).unwrap();
        io_uring_register_files(uring_fd, &[open_fd]).unwrap();
    }

    #[test]
    fn uring_register_io_slices() {
        let Some(uring_fd) = setup_io_poll_uring() else {
            return;
        };
        let mut buf1 = [0; 1024];
        let mut buf2 = [0; 1024];
        let io_slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
        io_uring_register_io_slices(uring_fd, &io_slices).unwrap();
    }

    #[test]
    fn uring_register_buffer() {
        let Some(uring_fd) = setup_io_poll_uring() else {
            return;
        };
        let buf1 = [0; 1024];
        io_uring_register_buf(uring_fd, &buf1).unwrap();
    }

    #[test]
    fn uring_setup_instance() {
        let uring = setup_io_uring(8, IoUringParamFlags::IORING_SETUP_SQPOLL);
        match uring {
            Ok(_u) => {}
            Err(e) => {
                #[cfg(target_arch = "aarch64")]
                if e.code.unwrap() == crate::error::Errno::ENOSYS {
                    return;
                }
                panic!("{e}")
            }
        }
    }
}
