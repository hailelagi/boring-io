use io_uring::types;
use io_uring::{opcode, IoUring};

use std::fs::File;
use std::io::Error;
use std::os::unix::io::AsRawFd;

fn read_readme() -> Result<(), Error> {
    let mut ring = IoUring::new(8)?;

    let fd = File::open("README.md")?;
    let mut buf = vec![0; 1024];

    let read_e = opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
        .build()
        .user_data(0x42);

    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    Ok(())
}
