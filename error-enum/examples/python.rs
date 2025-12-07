//! An example.

use error_enum::{error_type, ErrorEnum, SimpleSpan};

error_type! {
    /// Defined error type, will be generated as an `enum`.
    #[derive(Debug)]
    pub MyError {
        // You can use a `#[diag(kind = "...")]` to set the error kind of the subtree.
        #[diag(kind = "Warn")]
        {
            // You can use `#[diag(number = "...")]` to specify a number (prefix) to the subtree.
            #[diag(number = "0")]
            // You can use `#[diag(msg = "...")]` to specify the error message of the variant.
            #[diag(msg = "The token `{0}` is not formatted well.")]
            Unformatted(
                String,
                #[diag(span)]
                SimpleSpan,
            )
        },
        #[diag(kind = "Error")]
        {
            #[diag(number = "0")]
            {
                // Nested `#[diag(number = ...)]` will be concatenated,
                // so this error will have an error number "00".
                #[diag(number = "0")]
                #[diag(msg = "`{term}` is expected to be of type `{expected_ty}`, but is of type `{actual_ty}`.")]
                TypeError {
                    /// The term.
                    term: String,
                    /// Expected type.
                    expected_ty: String,
                    /// Actual type.
                    actual_ty: String,
                    /// Span of the term.
                    #[diag(span)]
                    span: SimpleSpan,
                }
            }
        },
    }
}

fn main() {
    let error = MyError::TypeError {
        term: "'1'".to_owned(),
        expected_ty: "int".to_owned(),
        actual_ty: "str".to_owned(),
        span: SimpleSpan::new("test.py", "print(1 + '1')", 10, 13),
    };

    assert_eq!(
        error.to_string(),
        "`'1'` is expected to be of type `int`, but is of type `str`."
    );
    assert_eq!(error.code(), "E00");

    #[cfg(feature = "annotate-snippets")]
    eprintln!("{}", error.fmt_as_annotate_snippets().unwrap());

    #[cfg(feature = "ariadne")]
    eprintln!("{}", error.fmt_as_ariadne_report().unwrap());

    #[cfg(feature = "miette")]
    eprintln!(
        "{}",
        error.fmt_as_miette_diagnostic_with(&miette::NarratableReportHandler::new())
    );
    #[cfg(feature = "miette")]
    eprintln!(
        "{}",
        error.fmt_as_miette_diagnostic_with(&miette::GraphicalReportHandler::new_themed(
            miette::GraphicalTheme::unicode()
        ))
    );
}
