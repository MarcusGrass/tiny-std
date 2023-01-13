#![no_std]
#![no_main]
#![allow(dead_code)]

use rusl::io_uring::{
    io_uring_enter, io_uring_register_buffers, io_uring_register_files, setup_io_uring,
};
use rusl::platform::{
    IoSliceMut, IoUringEnterFlags, IoUringParamFlags, IoUringSQEFlags, IoUringSubmissionQueueEntry,
    STDIN, STDOUT,
};
use unix_print::unix_eprintln;

/// Panic handler
#[panic_handler]
pub fn on_panic(info: &core::panic::PanicInfo) -> ! {
    unix_print::unix_eprintln!("Panicked {info}");
    tiny_std::process::exit(1)
}

#[no_mangle]
pub fn main() -> i32 {
    const PREFIX: &[u8; 6] = b"Echo: ";
    let mut uring = setup_io_uring(
        64,
        IoUringParamFlags::IORING_SETUP_SINGLE_ISSUER | IoUringParamFlags::IORING_SETUP_SQPOLL,
    )
    .unwrap();
    let mut stdin_buffer = [0u8; 1024];
    let stdin_buf_addr = core::ptr::addr_of_mut!(stdin_buffer) as u64;
    let mut stdout_buffer = [0u8; 1024];
    stdout_buffer[0..6].copy_from_slice(PREFIX);
    let stdout_buf_addr = core::ptr::addr_of_mut!(stdout_buffer) as u64;
    unsafe {
        io_uring_register_buffers(
            uring.fd,
            &[
                IoSliceMut::new(&mut stdin_buffer),
                IoSliceMut::new(&mut stdout_buffer),
            ],
        )
        .unwrap()
    };
    io_uring_register_files(uring.fd, &[STDIN, STDOUT]).unwrap();
    let mut last_read = 0;
    unsafe {
        let in_slot = uring.get_next_sqe_slot().unwrap();
        in_slot.write(IoUringSubmissionQueueEntry::new_readv_fixed(
            0,
            0,
            stdin_buf_addr,
            1024,
            STDIN as u64,
            IoUringSQEFlags::IOSQE_FIXED_FILE,
        ));
        uring.flush_submission_queue();
        io_uring_enter(uring.fd, 1, 1, IoUringEnterFlags::IORING_ENTER_GETEVENTS).unwrap();
        let cqe = uring.get_next_cqe().unwrap();
        if cqe.0.res < 0 {
            panic!("Got error on read {cqe:?}");
        }
        last_read = cqe.0.res as usize;
    }
    loop {
        unsafe {
            stdout_buffer[6..6 + last_read].copy_from_slice(&stdin_buffer[0..last_read]);
            let echo_slot = uring.get_next_sqe_slot().unwrap();
            echo_slot.write(IoUringSubmissionQueueEntry::new_writev_fixed(
                1,
                1,
                stdout_buf_addr,
                6 + last_read as u32,
                STDOUT as u64,
                IoUringSQEFlags::IOSQE_FIXED_FILE | IoUringSQEFlags::IOSQE_IO_LINK,
            ));
            let read_input_slot = uring.get_next_sqe_slot().unwrap();
            read_input_slot.write(IoUringSubmissionQueueEntry::new_readv_fixed(
                0,
                0,
                stdin_buf_addr,
                1024,
                STDIN as u64,
                IoUringSQEFlags::IOSQE_FIXED_FILE | IoUringSQEFlags::IOSQE_IO_LINK,
            ));
            uring.flush_submission_queue();
            io_uring_enter(
                uring.fd,
                2,
                2,
                IoUringEnterFlags::IORING_ENTER_GETEVENTS
                    | IoUringEnterFlags::IORING_ENTER_SQ_WAKEUP,
            )
            .unwrap();
            for _ in 0..2 {
                let cqe = uring.get_next_cqe().unwrap();
                if cqe.0.res < 0 {
                    panic!("Got bad res on cqe {cqe:?}");
                }
                match cqe.0.user_data as i32 {
                    STDIN => last_read = cqe.0.res as usize,
                    _ => {}
                }
            }
        }
    }
}
