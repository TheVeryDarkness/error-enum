//! Example demonstrating wrapping errors using error-enum crate.
#![expect(clippy::unwrap_used)]
#![expect(clippy::std_instead_of_core)]

use error_enum::ErrorType;
use std::{io, path::PathBuf};

#[derive(Debug, ErrorType)]
enum ReadIntError {
    #[diag(msg = "Failed to parse integer from string due to: {0}")]
    ParseIntError(std::num::ParseIntError),
    #[diag(msg = "Failed to read string due to: {0}")]
    IOError(io::Error),
    #[diag(msg = "{error}")]
    #[diag(help = "canonicalizing {path:?}")]
    CanonicalizeError {
        path: PathBuf,
        error: std::io::Error,
    },
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

    let canonicalize_error = ReadIntError::CanonicalizeError {
        path: PathBuf::from("path/to/file"),
        error: io::Error::new(io::ErrorKind::NotFound, "file not found"),
    };
    println!("CanonicalizeError: {}", canonicalize_error);
}
