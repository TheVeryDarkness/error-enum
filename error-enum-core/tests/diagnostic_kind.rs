//! Tests for built-in [`DiagnosticKind`] / [`Kind`].

use error_enum_core::{DiagnosticKind, Kind};

#[test]
fn builtin_code_prefix() {
    assert_eq!(Kind::Error.code_prefix(), "E");
    assert_eq!(Kind::Warn.code_prefix(), "W");
}

#[test]
#[cfg(feature = "annotate-snippets")]
fn builtin_annotate_snippets() {
    use annotate_snippets::snippet::AnnotationType;
    assert!(matches!(
        Kind::Error.as_annotate_snippets(),
        AnnotationType::Error
    ));
    assert!(matches!(
        Kind::Warn.as_annotate_snippets(),
        AnnotationType::Warning
    ));
}

#[test]
#[cfg(feature = "codespan-reporting")]
fn builtin_codespan() {
    use codespan_reporting::diagnostic::Severity;
    assert!(matches!(Kind::Error.as_codespan(), Severity::Error));
    assert!(matches!(Kind::Warn.as_codespan(), Severity::Warning));
}

#[test]
#[cfg(feature = "miette")]
fn builtin_miette() {
    assert!(matches!(Kind::Error.as_miette(), miette::Severity::Error));
    assert!(matches!(Kind::Warn.as_miette(), miette::Severity::Warning));
}
