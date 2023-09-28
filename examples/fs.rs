use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    pub FileSystemError
        E01 FileNotFound {path: PathBuf}
            "File {path:?} not found.",
}

fn main() {
    println!(
        "{}",
        FileSystemError::FileNotFound {
            path: "fs.rs".into()
        }
    )
}
