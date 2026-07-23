use super::test_error_type_derive;
use quote::quote;

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
                    #[diag(note("consider reformatting the token here"))]
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
                type Kind = ::error_enum::Kind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self::ParseIntError(..) => {
                            <::error_enum::Kind as ::core::default::Default>::default()
                        }
                        Self::IOError(..) => {
                            <::error_enum::Kind as ::core::default::Default>::default()
                        }
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self::ParseIntError(..) => ::error_enum::Cow::Borrowed("00"),
                        Self::IOError(..) => ::error_enum::Cow::Borrowed("01"),
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
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        Self::ParseIntError(_0) => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                            ::error_enum::format!("Failed to parse integer from string due to: {_0}")
                        )],
                        Self::IOError(_0, _1, _2) => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_0),
                            ::error_enum::format!("Failed to read string due to: {_2}")
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
                        Self::ParseIntError(_0) => ::error_enum::Box::new([].into_iter()),
                        Self::IOError(_0, _1, _2) => ::error_enum::Box::new(
                            [(
                                ::error_enum::format!("consider reformatting the token here"),
                                ::error_enum::vec1![(
                                    <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_1),
                                    ::error_enum::format!("consider reformatting the token here")
                                )],
                                ::error_enum::AdditionalKind::Note,
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
            #[diag(note("Got a string {0:?}"))]
            struct ReadIntError<'a>(
                #[diag(help("consider reformatting the token {0:?}"))]
                &'a str,
                std::io::Error,
                #[diag(span)]
                SimpleSpan,
                #[diag(note("consider reformatting the token {0:?}"))]
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
                type Kind = ::error_enum::Kind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self(..) => <::error_enum::Kind as ::core::default::Default>::default(),
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self(..) => ::error_enum::Cow::Borrowed(""),
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
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        Self(_0, _1, _2, _3) => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_2),
                            ::error_enum::format!("Failed to read an integer due to: {_1}")
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
                        Self(_0, _1, _2, _3) => ::error_enum::Box::new(
                            [
                                (
                                    ::error_enum::format!("Got a string {_0:?}"),
                                    ::error_enum::vec1![(
                                        <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                                        ::error_enum::format!("Got a string {_0:?}")
                                    )],
                                    ::error_enum::AdditionalKind::Note,
                                ),
                                (
                                    ::error_enum::format!("consider reformatting the token {_0:?}"),
                                    ::error_enum::vec1![(
                                        <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_0),
                                        ::error_enum::format!("consider reformatting the token {_0:?}")
                                    )],
                                    ::error_enum::AdditionalKind::Help,
                                ),
                                (
                                    ::error_enum::format!("consider reformatting the token {_0:?}"),
                                    ::error_enum::vec1![(
                                        <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(_3),
                                        ::error_enum::format!("consider reformatting the token {_0:?}")
                                    )],
                                    ::error_enum::AdditionalKind::Note,
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
            #[diag(help("due to: {error}"))]
            struct ParseIntError<'a> {
                #[diag(help("consider changing the string to an integer"))]
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
                type Kind = ::error_enum::Kind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self { .. } => <::error_enum::Kind as ::core::default::Default>::default(),
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self { .. } => ::error_enum::Cow::Borrowed(""),
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
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        #[allow(unused_variables)]
                        Self {
                            note_span,
                            error,
                            span,
                        } => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(span),
                            ::error_enum::format!("Failed to parse the string to an integer")
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
                        Self {
                            note_span,
                            error,
                            span,
                        } => ::error_enum::Box::new(
                            [
                                (
                                    ::error_enum::format!("due to: {error}"),
                                    ::error_enum::vec1![(
                                        <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                                        ::error_enum::format!("due to: {error}")
                                    )],
                                    ::error_enum::AdditionalKind::Help,
                                ),
                                (
                                    ::error_enum::format!("consider changing the string to an integer"),
                                    ::error_enum::vec1![(
                                        <::error_enum::SimpleSpan as ::core::convert::From<_>>::from(note_span),
                                        ::error_enum::format!("consider changing the string to an integer")
                                    )],
                                    ::error_enum::AdditionalKind::Help,
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

#[test]
fn custom_kind_type_and_expr() {
    test_error_type_derive(
        quote! {
            #[derive(Debug, ErrorType)]
            #[diag(kind_type = "MyKind")]
            #[diag(kind = MyKind::Bug)]
            #[diag(number = "01")]
            #[diag(msg = "boom")]
            struct Ice;
        },
        quote! {
            impl ::core::fmt::Display for Ice {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self => ::core::write!(f, "boom"),
                    }
                }
            }
            impl ::core::error::Error for Ice {}
            impl ::error_enum::ErrorType for Ice {
                type Span = ::error_enum::SimpleSpan;
                type Kind = MyKind;
                type Message = ::error_enum::String;
                type Label = ::error_enum::String;
                fn kind(&self) -> Self::Kind {
                    match self {
                        Self => MyKind::Bug,
                    }
                }
                fn number(&self) -> ::error_enum::Cow<'_, ::core::primitive::str> {
                    match self {
                        Self => ::error_enum::Cow::Borrowed("01"),
                    }
                }
                fn primary_span(&self) -> ::core::option::Option<::error_enum::SimpleSpan> {
                    match self {
                        Self => ::core::option::Option::None,
                    }
                }
                fn primary_message(&self) -> ::error_enum::String {
                    ::error_enum::format!("{self}")
                }
                fn primary_labels(
                    &self,
                ) -> ::error_enum::LabelVec1<::error_enum::SimpleSpan, ::error_enum::String> {
                    match self {
                        Self => ::error_enum::vec1![(
                            <::error_enum::SimpleSpan as ::core::default::Default>::default(),
                            ::error_enum::format!("boom")
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
                        Self => ::error_enum::Box::new([].into_iter()),
                    }
                }
            }
        },
    );
}
