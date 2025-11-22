use crate::ErrorEnum;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[track_caller]
fn test_error_type(tokens: TokenStream, expected: TokenStream) {
    let input: ErrorEnum = syn::parse2(tokens).unwrap();
    let output = input.into_token_stream();
    assert_eq!(output.to_string(), expected.to_string(), "Got:\n{}", output);
}

#[test]
fn basic() {
    test_error_type(
        quote! {
            FileSystemError {
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(code = 01)]
                    #[diag(msg = "{path} not found.")]
                    FileNotFound {path: std::path::Path},
                },
            }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- `E`: 错误"]
            #[doc = "  - `E01`(**FileNotFound**): {path} not found."]
            enum FileSystemError {
                #[doc = "`E01`: {path} not found."]
                #[doc(alias = "E01")]
                FileNotFound { path: std::path::Path },
            }
            impl ::core::fmt::Display for FileSystemError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => write!(f, "{path} not found."),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
        },
    );
}

#[test]
fn deep() {
    test_error_type(
        quote! {
            FileSystemError {
                #[diag(kind = "Error")]
                {
                    #[diag(code = 0)]
                    #[diag(msg = "文件错误")]
                    {
                        #[diag(code = 0)]
                        #[diag(msg = "无权限。")]
                        AccessDenied,
                    },
                },
            }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- `E`"]
            #[doc = "  - `E0`: 文件错误"]
            #[doc = "    - `E00`(**AccessDenied**): 无权限。"]
            enum FileSystemError {
                #[doc = "`E00`: 无权限。"]
                #[doc(alias = "E00")]
                AccessDenied,
            }
            impl ::core::fmt::Display for FileSystemError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::AccessDenied => write!(f, "无权限。"),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
        },
    );
}

#[test]
fn nested() {
    test_error_type(
        quote! {
            FileSystemError {
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(code = 01)]
                    #[diag(msg = "{0}")]
                    #[diag(nested)]
                    FileError (FileError),
                },
            }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- `E`: 错误"]
            #[doc = "  - `E01`(**FileError**): {0}"]
            enum FileSystemError {
                #[doc = "`E01`: {0}"]
                #[doc(alias = "E01")]
                FileError(FileError),
            }
            impl ::core::fmt::Display for FileSystemError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::FileError(_0) => write!(f, "{0}", _0),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
        },
    );
}
#[test]
fn escaped_braces_in_msg() {
    test_error_type(
        quote! {
            FileSystemError {
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(code = 01)]
                    #[diag(msg = "{{0}} not found.")]
                    FileNotFound (std::path::Path),
                },
            }
        },
        quote! {
            #[doc = "List of error variants:"]
            #[doc = "- `E`: 错误"]
            #[doc = "  - `E01`(**FileNotFound**): {{0}} not found."]
            enum FileSystemError {
                #[doc = "`E01`: {{0}} not found."]
                #[doc(alias = "E01")]
                FileNotFound(std::path::Path),
            }
            impl ::core::fmt::Display for FileSystemError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::FileNotFound(_0) => write!(f, "{{0}} not found."),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
        },
    );
}
