use clap::{value_parser, Arg, ArgAction, Command};
use io_uring::IoUring;
use std::io;

mod cat;
mod read_readme;

// use crate::cat::{get_completion_and_print, submit_read_request};

fn main() -> io::Result<()> {
    let m = Command::new("io_uring experiments")
        .arg(
            Arg::new("file")
                .action(ArgAction::Append)
                .value_parser(value_parser!(String))
                .short('f')
                .required(true),
        )
        .get_matches_from(vec!["file"]);

    let files: Vec<&str> = m
        .get_many::<String>("files")
        .expect("kitty cat")
        .map(|s: &_| s.as_str())
        .collect();

    let mut rq = IoUring::new(8)?;
    let mut cq = IoUring::new(8)?;

    /*
    for file_path in &files {
        submit_read_request(file_path, &mut rq)?;
        get_completion_and_print(&mut cq)?;
    }
    */

    Ok(())
}
