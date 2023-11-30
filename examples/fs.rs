use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    pub FileSystemError
        E "Errors." {
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
        FileSystemError::FileNotFound {
            path: "fs.rs".into()
        },
    );
    println!("{}", FileSystemError::NotAFile("target".into()),);
    println!("{}", FileSystemError::AccessDenied);
}
