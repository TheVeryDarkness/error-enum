use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::ErrorEnum;
use prettydiff::diff_lines;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[track_caller]
fn assert_eq_source(actual: &str, expected: &str) {
    if expected != actual {
        let diff = diff_lines(expected, actual);
        panic!(
            "---------- Source DIFF ----------\n{}\n--------- ACTUAL CODE ----------\n{}",
            diff, actual
        );
    }
}

fn format_str(source: &str) -> String {
    let path = if cfg!(target_os = "windows") {
        "rustfmt.exe"
    } else {
        "rustfmt"
    };
    let mut rustfmt = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = rustfmt.stdin.as_mut().unwrap();
    stdin.write_all(source.as_bytes()).unwrap();
    let output = rustfmt.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[track_caller]
fn test_error_type(tokens: TokenStream, expected: TokenStream) {
    let input: ErrorEnum = syn::parse2(tokens).unwrap();
    let output = input.into_token_stream();
    let output = format_str(&output.to_string());
    let expected = format_str(&expected.to_string());
    assert_eq_source(&output, &expected);
}

#[test]
fn basic() {
    test_error_type(
        quote! {
            #[derive(Debug)]
            FileSystemError {
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(number = "01")]
                    #[diag(msg = "{path} not found.")]
                    FileNotFound {path: std::path::Path},
                },
            }
        },
        quote! {
            #[derive(Debug)]
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
                        Self::FileNotFound { path } => ::core::write!(f, "{path} not found."),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
            impl ::error_enum::ErrorEnum for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::std::string::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self::FileNotFound { .. } => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self::FileNotFound { .. } => "01",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self::FileNotFound { .. } => "E01",
                    }
                }
                fn primary_span(&self) -> ::error_enum::SimpleSpan {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => {
                            <::error_enum::SimpleSpan as ::core::default::Default>::default()
                        }
                    }
                }
                fn primary_message(&self) -> ::std::string::String {
                    ::std::format!("{self}")
                }
                fn primary_label(&self) -> ::std::string::String {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => ::std::format!("{path} not found."),
                    }
                }
            }
        },
    );
}

#[test]
fn deep() {
    test_error_type(
        quote! {
            #[derive(Debug)]
            FileSystemError {
                #[diag(kind = "Error")]
                {
                    #[diag(number = "0")]
                    #[diag(msg = "文件错误")]
                    {
                        #[diag(number = "0")]
                        #[diag(msg = "无权限。")]
                        AccessDenied,
                    },
                },
            }
        },
        quote! {
            #[derive(Debug)]
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
                        Self::AccessDenied => ::core::write!(f, "无权限。"),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
            impl ::error_enum::ErrorEnum for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::std::string::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self::AccessDenied => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self::AccessDenied => "00",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self::AccessDenied => "E00",
                    }
                }
                fn primary_span(&self) -> ::error_enum::SimpleSpan {
                    match self {
                        Self::AccessDenied => <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                    }
                }
                fn primary_message(&self) -> ::std::string::String {
                    ::std::format!("{self}")
                }
                fn primary_label(&self) -> ::std::string::String {
                    match self {
                        Self::AccessDenied => ::std::format!("无权限。"),
                    }
                }
            }
        },
    );
}

#[test]
fn nested() {
    test_error_type(
        quote! {
            #[derive(Debug)]
            FileSystemError {
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(number = "01")]
                    #[diag(msg = "{0}")]
                    #[diag(nested)]
                    FileError (FileError),
                },
            }
        },
        quote! {
            #[derive(Debug)]
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
                        Self::FileError(_0) => ::core::write!(f, "{0}", _0),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
            impl ::error_enum::ErrorEnum for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::std::string::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self::FileError(..) => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self::FileError(..) => "01",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self::FileError(..) => "E01",
                    }
                }
                fn primary_span(&self) -> ::error_enum::SimpleSpan {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileError(_0) => {
                            <::error_enum::SimpleSpan as ::core::default::Default>::default()
                        }
                    }
                }
                fn primary_message(&self) -> ::std::string::String {
                    ::std::format!("{self}")
                }
                fn primary_label(&self) -> ::std::string::String {
                    match self {
                        Self::FileError(_0) => ::std::format!("{0}", _0),
                    }
                }
            }
        },
    );
}
#[test]
fn escaped_braces_in_msg() {
    test_error_type(
        quote! {
            #[derive(Debug)]
            FileSystemError {
                #[diag(kind = "Error")]
                #[diag(msg = "错误")]
                {
                    #[diag(number = "01")]
                    #[diag(msg = "{{0}} not found.")]
                    FileNotFound (std::path::Path),
                },
            }
        },
        quote! {
            #[derive(Debug)]
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
                        Self::FileNotFound(_0) => ::core::write!(f, "{{0}} not found."),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
            impl ::error_enum::ErrorEnum for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::std::string::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self::FileNotFound(..) => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self::FileNotFound(..) => "01",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self::FileNotFound(..) => "E01",
                    }
                }
                fn primary_span(&self) -> ::error_enum::SimpleSpan {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound(_0) => {
                            <::error_enum::SimpleSpan as ::core::default::Default>::default()
                        }
                    }
                }
                fn primary_message(&self) -> ::std::string::String {
                    ::std::format!("{self}")
                }
                fn primary_label(&self) -> ::std::string::String {
                    match self {
                        Self::FileNotFound(_0) => ::std::format!("{{0}} not found."),
                    }
                }
            }
        },
    );
}
