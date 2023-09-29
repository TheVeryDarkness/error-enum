use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    pub FileSystemError
        E "Errors." {
            0 "File-Related Errors." {
                0 FileNotFound {path: PathBuf}
                    "File `{path:?}` not found.",
                1 NotAFile {path: PathBuf}
                    "Path `{path:?}` does not point to a file.",
            }
        }
}

fn main() {
    println!(
        "{}",
        FileSystemError::FileNotFound {
            path: "fs.rs".into()
        }
    )
}
