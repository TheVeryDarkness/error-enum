use crate::ErrorEnum;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

fn test_error_type(tokens: TokenStream, expected: TokenStream) {
    let input: ErrorEnum = syn::parse2(tokens).unwrap();
    let output = input.into_token_stream();
    assert_eq!(output.to_string(), expected.to_string(), "Got:\n{}", output);
}

#[test]
fn basic() {
    test_error_type(
        quote! {
            FileSystemError
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(code = 01)]
                    #[diag(msg = "{path} not found.")]
                    FileNotFound {path: std::path::Path},
                }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- ``: 错误"]
            #[doc = "  - `01`(**FileNotFound**): {path} not found."]
            enum FileSystemError {
                FileNotFound { path : std::path::Path }
            }
        },
    );
}

#[test]
fn deep() {
    test_error_type(
        quote! {
            FileSystemError
                #[diag(kind = "Error")]
                {
                    #[diag(code = 0)]
                    #[diag(msg = "文件错误")]
                    {
                        #[diag(code = 0)]
                        #[diag(msg = "无权限。")]
                        AccessDenied,
                    }
                }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- ``: "]
            #[doc = "  - `0`: 文件错误"]
            #[doc = "    - `00`(**AccessDenied**): 无权限。"]
            enum FileSystemError { AccessDenied }
        },
    );
}

#[test]
fn nested() {
    test_error_type(
        quote! {
            FileSystemError
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(code = 01)]
                    #[diag(msg = "{0}")]
                    #[diag(nested)]
                    FileError (FileError),
                }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- ``: 错误"]
            #[doc = "  - `01`(**FileError**): {0}"]
            enum FileSystemError {
                FileError(FileError)
            }
        },
    );
}
