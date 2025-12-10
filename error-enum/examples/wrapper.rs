//! Example demonstrating wrapping errors using error-enum crate.
#![expect(clippy::unwrap_used)]

use error_enum::ErrorType;
use std::io;

#[derive(Debug, ErrorType)]
enum ReadIntError {
    #[diag(msg = "Failed to parse integer from string due to: {0}")]
    ParseIntError(std::num::ParseIntError),
    #[diag(msg = "Failed to read string due to: {0}")]
    IOError(io::Error),
}

#[derive(Debug, ErrorType)]
#[diag(msg = "Failed to read string due to: {0}")]
struct IOError(io::Error);

fn main() {
    let parse_error = ReadIntError::ParseIntError("abc".parse::<i32>().unwrap_err());
    println!("ParseIntError: {}", parse_error);

    let io_error = ReadIntError::IOError(io::Error::other("disk error"));
    println!("IOError: {}", io_error);

    let simple_io_error = IOError(io::Error::new(io::ErrorKind::NotFound, "file not found"));
    println!("Simple IOError: {}", simple_io_error);
}
