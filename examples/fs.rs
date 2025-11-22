use error_enum::error_type;
use std::path::PathBuf;

error_type! {
    #[derive(Debug)]
    pub FileSystemError {
        #[diag(kind = "Error")]
        #[diag(msg = "Errors.")]
        {
            #[diag(code = 0)]
            #[diag(msg = "File Kind-Related Errors.")]
            {
                #[diag(code = 0)]
                #[diag(msg = "File {path:?} Not Found")]
                FileNotFound {path: PathBuf},
                #[diag(code = 1)]
                #[diag(msg = "Path {0:?} does not point to a file.")]
                NotAFile (PathBuf),
            },
            #[diag(code = 1)]
            #[diag(msg = "Access-Related Errors.")]
            {
                #[diag(code = 1)]
                #[diag(msg = "Access Denied.")]
                AccessDenied,
            },
        },
        #[diag(kind = "Warn")]
        #[diag(msg = "Warnings.")]
        {
            #[diag(code = 0)]
            #[diag(msg = "File Content-Related Warnings.")]
            {
                #[diag(code = 0)]
                #[diag(msg = "File {path:?} is too big. Consider read it with stream or in parts.")]
                FileTooLarge {path: PathBuf},
            },
        },
    }
}

fn main() {
    assert_eq!(
        FileSystemError::FileNotFound {
            path: "fs.rs".into()
        }
        .to_string(),
        "File \"fs.rs\" Not Found",
    );
    assert_eq!(
        FileSystemError::NotAFile("target".into()).to_string(),
        "Path \"target\" does not point to a file."
    );
    assert_eq!(FileSystemError::AccessDenied.to_string(), "Access Denied.");
    assert_eq!(
        FileSystemError::FileTooLarge {
            path: "data.json".into(),
        }
        .to_string(),
        "File \"data.json\" is too big. Consider read it with stream or in parts."
    );
}
