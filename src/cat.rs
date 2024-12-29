use io_uring::{cqueue, opcode, squeue, IoUring};
use std::alloc::{alloc, Layout};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use libc::iovec;
use std::ptr;
use std::slice;

const BLOCK_SIZE: usize = 1024;

struct FileInfo {
    file_sz: u64,
    iovecs: Vec<iovec>,
}

fn get_file_size(file: &File) -> io::Result<u64> {
    let metadata = file.metadata()?;
    Ok(metadata.len())
}

fn output_to_console(buf: &[u8]) {
    io::stdout().write_all(buf).unwrap();
}

fn get_completion_and_print(ring: &mut IoUring) -> io::Result<()> {
    let cqe = ring.wait_for_cqe()?;
    let res = cqe.result();
    if res < 0 {
        eprintln!("Async readv failed.");
        return Err(io::Error::from_raw_os_error(-res));
    }

    let fi: &FileInfo = unsafe { &*(cqe.user_data() as *const FileInfo) };
    let blocks = (fi.file_sz as usize + BLOCK_SIZE - 1) / BLOCK_SIZE;
    for i in 0..blocks {
        let iovec = &fi.iovecs[i];
        let buf = unsafe { slice::from_raw_parts(iovec.iov_base as *const u8, iovec.iov_len) };
        output_to_console(buf);
    }

    ring.cq().advance(1);
    Ok(())
}

fn submit_read_request(file_path: &str, ring: &mut IoUring) -> io::Result<()> {
    let file = File::open(file_path)?;
    let file_sz = get_file_size(&file)?;
    let mut bytes_remaining = file_sz;
    let mut offset = 0;
    let mut current_block = 0;
    let blocks = (file_sz as usize + BLOCK_SIZE - 1) / BLOCK_SIZE;

    let mut fi = FileInfo {
        file_sz,
        iovecs: Vec::with_capacity(blocks),
    };

    while bytes_remaining > 0 {
        let bytes_to_read = std::cmp::min(bytes_remaining, BLOCK_SIZE as u64);
        let layout = Layout::from_size_align(BLOCK_SIZE, BLOCK_SIZE).unwrap();
        let buf = unsafe { alloc(layout) };

        fi.iovecs.push(iovec {
            iov_base: buf as *mut _,
            iov_len: bytes_to_read as usize,
        });

        current_block += 1;
        bytes_remaining -= bytes_to_read;
        offset += bytes_to_read;
    }

    let sqe = ring.next_sqe().unwrap();
    let readv = opcode::Readv::new(
        squeue::Target::Fd(file.as_raw_fd()),
        fi.iovecs.as_ptr(),
        blocks as u32,
    )
    .build()
    .user_data(&fi as *const _ as u64);

    unsafe {
        sqe.prep(&readv);
    }

    ring.submit()?;
    Ok(())
}
