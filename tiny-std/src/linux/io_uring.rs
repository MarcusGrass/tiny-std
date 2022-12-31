use alloc::vec;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;

use sc::syscall;
use unix_print::unix_eprintln;
use rusl::io_uring::io_uring_register_io_slices;

use rusl::platform::{IoSliceMut, IoUringCompletionQueueEntry, IoUringParamFlags, IoUringParams, IoUringSubmissionQueueEntry, MapAdditionalFlags, MapRequiredFlag, MemoryProtection};
use rusl::unistd::mmap;

use crate::error::Result;
use crate::unix::fd::{OwnedFd, RawFd};

#[derive(Debug)]
pub struct UringDriver<const N: usize> {
    uring_fd: OwnedFd,
    params: IoUringParams,
    submission_queue: UringSubmissionQueue,
    completion_queue: UringCompletionQueue,
    buffers: [Vec<u8>; N],
    file_writes: Vec<UringFileWrite>,
}

#[derive(Debug)]
pub struct UringFileWrite {
    fd: RawFd,
    buf_ind: usize,
}

#[derive(Debug)]
struct UringSubmissionQueue {
    ring_size: usize,
    ring_ptr: usize,
    kernel_head: usize,
    kernel_tail: usize,
    kernel_flags: usize,
    kernel_dropped: usize,
    kernel_array: usize,
    head: usize,
    tail: usize,
    ring_mask: u32,
    ring_entries: u32,
    entries: *mut IoUringSubmissionQueueEntry,
}

#[derive(Debug)]
struct UringCompletionQueue {
    ring_size: usize,
    ring_ptr: usize,
    kernel_head: usize,
    kernel_tail: usize,
    kernel_flags: usize,
    kernel_overflow: usize,
    ring_mask: u32,
    ring_entries: u32,
    entries: usize,
}

impl<const N: usize> UringDriver<N> {
    /// Create a uring driver that will only be accessed by a single thread,
    /// the kernel does some optimizations here.
    /// # Errors
    /// Syscall to setup `uring` fails
    /// # Safety
    /// If used in a multi-threaded environment, anything could happen
    pub unsafe fn new_single_threaded(mut buffers: [Vec<u8>; N]) -> Result<UringDriver<N>> {
        let mut params = IoUringParams::new(
            IoUringParamFlags::IORING_SETUP_SQPOLL | IoUringParamFlags::IORING_SETUP_SINGLE_ISSUER,
        );
        let fd = rusl::io_uring::io_uring_setup(N as u32, &mut params)?;
        let mut cq_size = core::mem::size_of::<IoUringCompletionQueueEntry>();
        let mut sq_ring_sz = params.0.sq_off.array as usize
            + params.0.sq_entries as usize * core::mem::size_of::<u32>();
        let mut cq_ring_sz = (params.0.cq_off.cqes + params.0.cq_entries) as usize * cq_size;
        if params.0.features & 1 != 0 {
            cq_ring_sz = sq_ring_sz;
        }
        let sq_ring_ptr = rusl::unistd::mmap(
            None,
            NonZeroUsize::new(sq_ring_sz).unwrap(),
            MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
            MapRequiredFlag::MapShared,
            MapAdditionalFlags::MAP_POPULATE,
            Some(fd),
            0,
        )?;
        // TODO: Use constant, 1 is SINGLE_MMAP
        let cq_ring_ptr = if params.0.features & 1 != 0 {
            sq_ring_ptr
        } else {
            mmap(
                None,
                NonZeroUsize::new(cq_ring_sz).unwrap(),
                MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
                MapRequiredFlag::MapShared,
                MapAdditionalFlags::MAP_POPULATE,
                Some(fd),
                0,
            )?
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
        let sqes = mmap(
            None,
            NonZeroUsize::new(size * params.0.sq_entries as usize).unwrap(),
            MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
            MapRequiredFlag::MapShared,
            MapAdditionalFlags::MAP_POPULATE,
            Some(fd),
            0,
        )?;
        let cq_khead = cq_ring_ptr + params.0.cq_off.head as usize;
        let cq_ktail = cq_ring_ptr + params.0.cq_off.tail as usize;
        let cq_koverflow = cq_ring_ptr + params.0.cq_off.overflow as usize;
        let cq_cqes = cq_ring_ptr + params.0.cq_off.cqes as usize;
        let cq_kflags = if params.0.cq_off.flags != 0 {
            cq_ring_ptr + params.0.cq_off.flags as usize
        } else {
            0
        };
        let sq_ring_mask = *((sq_ring_ptr + params.0.sq_off.ring_mask as usize) as *const u32);
        let sq_ring_entries =
            *((sq_ring_ptr + params.0.sq_off.ring_entries as usize) as *const u32);
        let cq_ring_mask = *((cq_ring_ptr + params.0.cq_off.ring_mask as usize) as *const u32);
        let cq_ring_entries =
            *((cq_ring_ptr + params.0.cq_off.ring_entries as usize) as *const u32);
        let mut slices = Vec::with_capacity(N);
        for buf in &mut buffers {
            slices.push(IoSliceMut::new(buf.as_mut_slice()));
        }
        io_uring_register_io_slices(fd, &slices)?;

        Ok(Self {
            uring_fd: OwnedFd(fd),
            params,
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
                entries: sqes as _,
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
            buffers,
            file_writes: vec![],
        })
    }

    unsafe fn add_file(&mut self, fd: RawFd) -> Result<()> {
        let next_buf_ptr = self.buffers[0].as_ptr() as u64;
        let next_entry = self.get_next_entry().unwrap();
        next_entry.0.flags |= 1 << 0;
        next_entry.0.fd = 0 as i32;
        next_entry.0.opcode = 4;
        next_entry.0.user_data = 15;
        next_entry.0.__bindgen_anon_2.addr = next_buf_ptr;
        let res = syscall!(IO_URING_REGISTER, self.uring_fd.0, 2, [fd].as_ptr(), 1);
        if res > usize::MAX - 256 {
            unix_eprintln!("\nAdd file res {}", 0 - res as i32);
        }
        unix_eprintln!("Res {res}");
        Ok(())
    }

    #[inline]
    unsafe fn get_next_entry(&mut self) -> Option<&mut IoUringSubmissionQueueEntry> {
        let next = self.submission_queue.tail + 1;
        let mut shift = 0;
        // TODO: Use constant, 10 is 128bit SQE
        if self.params.0.flags & 1 << 10 != 0 {
            shift += 1;
        }
        // Should read atomic
        let head = *(self.submission_queue.kernel_head as *const u32) as usize;
        if next - head <= self.submission_queue.ring_entries as usize {
            let ind =
                (self.submission_queue.tail & self.submission_queue.ring_mask as usize) << shift;
            let sqe = self.submission_queue.entries.add(ind);
            self.submission_queue.tail = next;
            Some(&mut *sqe)
        } else {
            None
        }
    }
}

impl<const N: usize> Drop for UringDriver<N> {
    fn drop(&mut self) {
        // TODO: Unmap rings, watch out for the double free on single mmap
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use rusl::platform::IoSliceMut;

    use crate::fs::File;
    use crate::linux::io_uring::UringDriver;
    use crate::unix::fd::AsRawFd;

    #[test]
    fn instantiate_uring_get_next() {
        unsafe {
            let buf = vec![0; 1024];
            let mut driver = UringDriver::new_single_threaded([buf]).unwrap();
            for i in 0..1 {
                assert!(driver.get_next_entry().is_some());
            }
            assert!(driver.get_next_entry().is_none());
        }
    }

    #[test]
    fn instantiate_uring_add_file() {
        unsafe {
            let file = File::open("test-files/fs/test1.txt\0").unwrap();
            let buf = vec![0; 1024];
            let mut driver = UringDriver::new_single_threaded([buf]).unwrap();
            driver.add_file(file.as_raw_fd()).unwrap();
        }
    }
}
