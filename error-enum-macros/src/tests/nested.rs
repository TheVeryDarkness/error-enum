use super::test_error_type;
use quote::quote;

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
                    #[diag(nested)]
                    FileError (FileError),
                },
            }
        },
        quote! {
            #[derive(Debug)]
            #[doc = "List of error variants:"]
            #[doc = "- `E`: 错误"]
            #[doc = "  - `E01`(**FileError**)"]
            enum FileSystemError {
                #[doc = "`E01`"]
                #[doc(alias = "E01")]
                FileError(FileError),
            }
            impl ::core::fmt::Display for FileSystemError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::FileError(inner) => ::core::write!(f, "{}", inner),
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
                        Self::FileError(inner) => {
                            let __kind = ::error_enum::ErrorType::kind(inner);
                            ::core::debug_assert_eq!(
                                ::error_enum::DiagnosticKind::code_prefix(
                                    &(::error_enum::Kind::Error)
                                ),
                                ::error_enum::DiagnosticKind::code_prefix(&__kind),
                            );
                            __kind
                        }
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::FileError(inner) => ::error_enum::Cow::Owned(::error_enum::format!(
                            "{}{}",
                            "01",
                            ::error_enum::ErrorType::number(inner)
                        )),
                    }
                }
                fn code(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::FileError(inner) => {
                            let __kind = ::error_enum::ErrorType::kind(inner);
                            ::error_enum::Cow::Owned(::error_enum::format!(
                                "{}{}{}",
                                ::error_enum::DiagnosticKind::code_prefix(&__kind),
                                "01",
                                ::error_enum::ErrorType::number(inner)
                            ))
                        }
                    }
                }
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        Self::FileError(inner) => ::error_enum::ErrorType::primary_span(inner),
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        Self::FileError(inner) => ::error_enum::ErrorType::primary_labels(inner),
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
                        Self::FileError(inner) => ::error_enum::ErrorType::additional(inner),
                    }
                }
            }
        },
    );
}
