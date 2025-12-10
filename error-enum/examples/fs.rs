//! Example of defining file system related errors and warnings with [`error_type!`] macro.

use error_enum::{error_type, ErrorType};
use std::path::PathBuf;

error_type! {
    #[derive(Debug)]
    pub FileSystemError {
        #[diag(kind = "Error")]
        #[diag(msg = "Errors.")]
        {
            #[diag(number = "0")]
            #[diag(msg = "File Kind-Related Errors.")]
            {
                #[diag(number = "0")]
                #[diag(msg = "File {path:?} Not Found")]
                FileNotFound {
                    /// File path
                    path: PathBuf
                },
                #[diag(number = "1")]
                #[diag(msg = "Path {0:?} does not point to a file.")]
                NotAFile (PathBuf),
            },
            #[diag(number = "1")]
            #[diag(msg = "Access-Related Errors.")]
            {
                #[diag(number = "1")]
                #[diag(msg = "Access Denied.")]
                AccessDenied,
            },
        },
        #[diag(kind = "Warn")]
        #[diag(msg = "Warnings.")]
        {
            #[diag(number = "0")]
            #[diag(msg = "File Content-Related Warnings.")]
            {
                #[diag(number = "0")]
                #[diag(msg = "File {path:?} is too big. Consider read it with stream or in parts.")]
                FileTooLarge {
                    /// File path
                    path: PathBuf
                },
            },
        },
    }
}

#[track_caller]
fn test_error(err: &FileSystemError, expected: &str, code: &str)
where
    FileSystemError: ErrorType,
{
    let msg = err.to_string();
    assert_eq!(msg, expected, "Got message: {}", msg);
    assert_eq!(err.code(), code, "Got code: {}", err.code());
}

fn main() {
    test_error(
        &FileSystemError::FileNotFound {
            path: "fs.rs".into(),
        },
        "File \"fs.rs\" Not Found",
        "E00",
    );
    test_error(
        &FileSystemError::NotAFile("target".into()),
        "Path \"target\" does not point to a file.",
        "E01",
    );
    test_error(&FileSystemError::AccessDenied, "Access Denied.", "E11");
    test_error(
        &FileSystemError::FileTooLarge {
            path: "data.json".into(),
        },
        "File \"data.json\" is too big. Consider read it with stream or in parts.",
        "W00",
    );
}
