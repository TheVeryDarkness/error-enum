use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    #[derive(Debug)]
    pub FileSystemError
        #[diag(kind = "Error")]
        #[diag(msg = "Errors.")]
        #[diag(nested)]
        {
            #[diag(code = 0)]
            #[diag(msg = "{0}")]
            FileError (FileError),
        },
}

error_type! {
    #[derive(Debug)]
    pub FileError
        #[diag(kind = "Error")]
        #[diag(msg = "Errors.")]
        {
            #[diag(code = 0)]
            #[diag(msg = "File-Related Errors.")]
            {
                #[diag(code = 0)]
                #[diag(msg = "File {path:?} not found.")]
                FileNotFound {path: PathBuf},
                #[diag(code = 1)]
                #[diag(msg = "Path {0:?} does not point to a file.")]
                NotAFile (PathBuf),
            },
            #[diag(code = 1)]
            #[diag(msg = "Access Denied.")]
            {
                #[diag(code = 0)]
                #[diag(msg = "Access Denied.")]
                AccessDenied,
            },
        },
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
