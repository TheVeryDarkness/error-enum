#![expect(clippy::unwrap_used, clippy::panic)]

use crate::ErrorEnum;
use prettydiff::diff_lines;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use syn::DeriveInput;

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
#[track_caller]
fn test_error_type_derive(tokens: TokenStream, expected: TokenStream) {
    let input: DeriveInput = syn::parse2(tokens).unwrap();
    let input = ErrorEnum::try_from(input).unwrap();
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
            impl ::error_enum::ErrorType for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
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
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => ::core::option::Option::None,
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => ::error_enum::format!("{path} not found."),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => ::error_enum::Box::new([].into_iter()),
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
            impl ::error_enum::ErrorType for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
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
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        Self::AccessDenied => ::core::option::Option::None,
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        Self::AccessDenied => ::error_enum::format!("无权限。"),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        Self::AccessDenied => ::error_enum::Box::new([].into_iter()),
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
                        Self::FileError(_0) => ::core::write!(f, "{_0}"),
                    }
                }
            }
            impl ::core::error::Error for FileSystemError {}
            impl ::error_enum::ErrorType for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
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
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileError(_0) => ::core::option::Option::None,
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        Self::FileError(_0) => ::error_enum::format!("{_0}"),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        Self::FileError(_0) => ::error_enum::Box::new([].into_iter()),
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
            impl ::error_enum::ErrorType for FileSystemError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
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
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound(_0) => ::core::option::Option::None,
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        Self::FileNotFound(_0) => ::error_enum::format!("{{0}} not found."),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        Self::FileNotFound(_0) => ::error_enum::Box::new([].into_iter()),
                    }
                }
            }
        },
    );
}

#[test]
fn test_error_type_with_derive_input() {
    test_error_type_derive(
        quote! {
            #[derive(Debug, ErrorType)]
            enum ReadIntError {
                #[diag(number = "00")]
                #[diag(msg = "Failed to parse integer from string due to: {0}")]
                ParseIntError(std::num::ParseIntError),
                #[diag(number = "01")]
                #[diag(msg = "Failed to read string due to: {2}")]
                IOError(
                    #[diag(span)]
                    SimpleSpan,
                    #[diag(note = "consider reformatting the token here")]
                    SimpleSpan,
                    std::io::Error,
                ),
            }
        },
        quote! {
            impl ::core::fmt::Display for ReadIntError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::ParseIntError(_0) => {
                            ::core::write!(f, "Failed to parse integer from string due to: {_0}")
                        }
                        Self::IOError(_0, _1, _2) => ::core::write!(f, "Failed to read string due to: {_2}"),
                    }
                }
            }
            impl ::core::error::Error for ReadIntError {}
            impl ::error_enum::ErrorType for ReadIntError {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self::ParseIntError(..) => ::error_enum::Kind::Error,
                        Self::IOError(..) => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self::ParseIntError(..) => "00",
                        Self::IOError(..) => "01",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self::ParseIntError(..) => "E00",
                        Self::IOError(..) => "E01",
                    }
                }
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        #[allow(unused_variables)]
                        Self::ParseIntError(_0) => ::core::option::Option::None,
                        #[allow(unused_variables)]
                        Self::IOError(_0, _1, _2) => {
                            ::core::option::Option::Some(<::error_enum::SimpleSpan as ::core::convert::From<
                                _,
                            >>::from(_0))
                        }
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        Self::ParseIntError(_0) => {
                            ::error_enum::format!("Failed to parse integer from string due to: {_0}")
                        }
                        Self::IOError(_0, _1, _2) => ::error_enum::format!("Failed to read string due to: {_2}"),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        Self::ParseIntError(_0) => ::error_enum::Box::new([].into_iter()),
                        Self::IOError(_0, _1, _2) => ::error_enum::Box::new(
                            [(
                                ::core::option::Option::Some(
                                    <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_1),
                                ),
                                ::error_enum::format!("consider reformatting the token here"),
                                ::error_enum::format!("consider reformatting the token here"),
                            )]
                            .into_iter(),
                        ),
                    }
                }
            }
        },
    );
    test_error_type_derive(
        quote! {
            #[derive(Debug, ErrorType)]
            #[diag(msg = "Failed to read an integer due to: {1}")]
            #[diag(note = "Got a string {0:?}")]
            struct ReadIntError<'a>(
                #[diag(help = "consider reformatting the token {0:?}")]
                &'a str,
                std::io::Error,
                #[diag(span)]
                SimpleSpan,
                #[diag(note = "consider reformatting the token {0:?}")]
                SimpleSpan,
            );
        },
        quote! {
            impl<'a> ::core::fmt::Display for ReadIntError<'a> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self(_0, _1, _2, _3) => ::core::write!(f, "Failed to read an integer due to: {_1}"),
                    }
                }
            }
            impl<'a> ::core::error::Error for ReadIntError<'a> {}
            impl<'a> ::error_enum::ErrorType for ReadIntError<'a> {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self(..) => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self(..) => "",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self(..) => "E",
                    }
                }
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        #[allow(unused_variables)]
                        Self(_0, _1, _2, _3) => {
                            ::core::option::Option::Some(<::error_enum::SimpleSpan as ::core::convert::From<
                                _,
                            >>::from(_2))
                        }
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        Self(_0, _1, _2, _3) => ::error_enum::format!("Failed to read an integer due to: {_1}"),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        Self(_0, _1, _2, _3) => ::error_enum::Box::new(
                            [
                                (
                                    ::core::option::Option::None,
                                    ::error_enum::format!("Got a string {_0:?}"),
                                    ::error_enum::format!("Got a string {_0:?}"),
                                ),
                                (
                                    ::core::option::Option::Some(
                                        <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_3),
                                    ),
                                    ::error_enum::format!("consider reformatting the token {_0:?}"),
                                    ::error_enum::format!("consider reformatting the token {_0:?}"),
                                ),
                                (
                                    ::core::option::Option::Some(
                                        <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_0),
                                    ),
                                    ::error_enum::format!("consider reformatting the token {_0:?}"),
                                    ::error_enum::format!("consider reformatting the token {_0:?}"),
                                ),
                            ]
                            .into_iter(),
                        ),
                    }
                }
            }
        },
    );
    test_error_type_derive(
        quote! {
            #[derive(Debug, ErrorType)]
            #[diag(msg = "Failed to parse the string to an integer")]
            #[diag(help = "due to: {error}")]
            struct ParseIntError<'a> {
                #[diag(help = "consider changing the string to an integer")]
                note_span: SimpleSpan,
                error: std::num::ParseIntError,
                #[diag(span)]
                span: SimpleSpan,
            }
        },
        quote! {
            impl<'a> ::core::fmt::Display for ParseIntError<'a> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #[allow(unused_variables)]
                        Self {
                            note_span,
                            error,
                            span,
                        } => ::core::write!(f, "Failed to parse the string to an integer"),
                    }
                }
            }
            impl<'a> ::core::error::Error for ParseIntError<'a> {}
            impl<'a> ::error_enum::ErrorType for ParseIntError<'a> {
                type Span = ::error_enum::SimpleSpan;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> ::error_enum::Kind {
                    match self {
                        Self { .. } => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> &::core::primitive::str {
                    match self {
                        Self { .. } => "",
                    }
                }
                fn code(&self) -> &::core::primitive::str {
                    match self {
                        Self { .. } => "E",
                    }
                }
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        #[allow(unused_variables)]
                        Self {
                            note_span,
                            error,
                            span,
                        } => {
                            ::core::option::Option::Some(<::error_enum::SimpleSpan as ::core::convert::From<
                                _,
                            >>::from(span))
                        }
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_label(&self) -> ::error_enum::String {
                    match self {
                        #[allow(unused_variables)]
                        Self {
                            note_span,
                            error,
                            span,
                        } => ::error_enum::format!("Failed to parse the string to an integer"),
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::core::option::Option<::error_enum::SimpleSpan>,
                            ::error_enum::String,
                            ::error_enum::String,
                        ),
                    >,
                > {
                    match self {
                        #[allow(unused_variables)]
                        Self {
                            note_span,
                            error,
                            span,
                        } => ::error_enum::Box::new(
                            [
                                (
                                    ::core::option::Option::None,
                                    ::error_enum::format!("due to: {error}"),
                                    ::error_enum::format!("due to: {error}"),
                                ),
                                (
                                    ::core::option::Option::Some(
                                        <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(note_span),
                                    ),
                                    ::error_enum::format!("consider changing the string to an integer"),
                                    ::error_enum::format!("consider changing the string to an integer"),
                                ),
                            ]
                            .into_iter(),
                        ),
                    }
                }
            }
        },
    );
}
