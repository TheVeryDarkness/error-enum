use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    pub FileSystemError
        #[color="red"]
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
        #[color=214]
        W "Warnings." {
            0 "File-Related Errors." {
                0 FileTooBig {path: PathBuf}
                    "File {path:?} is too big. Consider read it with stream or in parts.",
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
    println!(
        "{}",
        FileSystemError::FileTooBig {
            path: "data.json".into()
        }
    );
}
