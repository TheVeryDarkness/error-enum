use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    pub FileSystemError
        #[nested]
        Error "Errors." {
            0 FileError (FileError)
                "{0}",
        }
}

error_type! {
    pub FileError
        Error "Errors." {
            0 "File-Related Errors." {
                0 FileNotFound {path: PathBuf}
                    "File {path:?} not found.",
                1 NotAFile (PathBuf)
                    "Path {0:?} does not point to a file.",
            }
            1 "Access Denied." {
                0 AccessDenied
                    "Access Denied.",
            }
        }
}

fn main() {
    println!(
        "{}",
        FileSystemError::FileError(FileError::FileNotFound {
            path: "fs.rs".into()
        }),
    );
    println!(
        "{}",
        FileSystemError::FileError(FileError::NotAFile("target".into()),)
    );
    println!("{}", FileSystemError::FileError(FileError::AccessDenied));
}
