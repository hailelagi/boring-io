use io_uring::{opcode, types, IoUring};
use std::os::unix::io::AsRawFd;
use std::{fs, io};
use clap::{App, Arg};

mod cat;
mod read_readme;

fn main() -> io::Result<()> {
    let matches = App::new("io_uring experiments")
        .version("0.0.1")
        .about("explore the io_uring interface")
        .arg(
            Arg::with_name("files")
                .multiple(true)
                .required(true)
                .help("Files to read"),
        )
        .get_matches();

    let files: Vec<&str> = matches.values_of("files").unwrap().collect();
    let mut ring = IoUring::new(8)?;

    for file_path in &files {
        submit_read_request(file_path, &mut ring)?;
        get_completion_and_print(&mut ring)?;
    }

    Ok(())
}
