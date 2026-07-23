//! Custom [`DiagnosticKind`] example.

use error_enum::{error_type, DiagnosticKind, ErrorType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MyKind {
    #[default]
    Bug,
    Lint,
}

impl DiagnosticKind for MyKind {
    fn code_prefix(&self) -> &str {
        match self {
            MyKind::Bug => "B",
            MyKind::Lint => "L",
        }
    }

    #[cfg(feature = "annotate-snippets")]
    fn as_annotate_snippets(&self) -> annotate_snippets::snippet::AnnotationType {
        match self {
            MyKind::Bug => annotate_snippets::snippet::AnnotationType::Error,
            MyKind::Lint => annotate_snippets::snippet::AnnotationType::Warning,
        }
    }

    #[cfg(feature = "ariadne")]
    fn as_ariadne(&self) -> ariadne::ReportKind<'static> {
        match self {
            MyKind::Bug => ariadne::ReportKind::Error,
            MyKind::Lint => ariadne::ReportKind::Warning,
        }
    }

    #[cfg(feature = "codespan-reporting")]
    fn as_codespan(&self) -> codespan_reporting::diagnostic::Severity {
        match self {
            MyKind::Bug => codespan_reporting::diagnostic::Severity::Bug,
            MyKind::Lint => codespan_reporting::diagnostic::Severity::Warning,
        }
    }

    #[cfg(feature = "miette")]
    fn as_miette(&self) -> miette::Severity {
        match self {
            MyKind::Bug => miette::Severity::Error,
            MyKind::Lint => miette::Severity::Warning,
        }
    }
}

error_type! {
    #[derive(Debug)]
    #[diag(kind_type = "MyKind")]
    CustomKindError {
        {
            #[diag(kind = MyKind::Bug)]
            #[diag(number = "01")]
            #[diag(msg = "internal compiler failure")]
            Ice,
            #[diag(kind = MyKind::Lint)]
            #[diag(number = "02")]
            #[diag(msg = "prefer the other style")]
            StyleHint,
        },
    }
}

fn main() {
    let ice = CustomKindError::Ice;
    assert_eq!(ice.code().as_ref(), "B01");
    assert_eq!(ice.kind(), MyKind::Bug);

    let hint = CustomKindError::StyleHint;
    assert_eq!(hint.code().as_ref(), "L02");
    assert_eq!(hint.kind(), MyKind::Lint);
    println!("{ice} ({})", ice.code());
}
