use super::test_error_type;
use quote::quote;

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
                type Kind = ::error_enum::Kind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self::FileNotFound { .. } => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::FileNotFound { .. } => ::error_enum::Cow::Borrowed("01"),
                    }
                }
                fn code(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::FileNotFound { .. } => ::error_enum::Cow::Borrowed("E01"),
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
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        #[allow(unused_variables)]
                        Self::FileNotFound { path } => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                            ::error_enum::format!("{path} not found.")
                        )],
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::error_enum::String,
                            ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String>,
                            ::error_enum::AdditionalKind,
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
                type Kind = ::error_enum::Kind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self::AccessDenied => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::AccessDenied => ::error_enum::Cow::Borrowed("00"),
                    }
                }
                fn code(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::AccessDenied => ::error_enum::Cow::Borrowed("E00"),
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
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        Self::AccessDenied => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                            ::error_enum::format!("无权限。")
                        )],
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::error_enum::String,
                            ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String>,
                            ::error_enum::AdditionalKind,
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
                type Kind = ::error_enum::Kind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self::FileNotFound(..) => ::error_enum::Kind::Error,
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::FileNotFound(..) => ::error_enum::Cow::Borrowed("01"),
                    }
                }
                fn code(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::FileNotFound(..) => ::error_enum::Cow::Borrowed("E01"),
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
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        Self::FileNotFound(_0) => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                            ::error_enum::format!("{{0}} not found.")
                        )],
                    }
                }
                fn additional(
                    &self,
                ) -> ::error_enum::Box<
                    dyn ::core::iter::Iterator<
                        Item = (
                            ::error_enum::String,
                            ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String>,
                            ::error_enum::AdditionalKind,
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
