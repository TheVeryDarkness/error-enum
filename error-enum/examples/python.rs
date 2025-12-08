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
            #[diag(label = "consider reformatting the token")]
            MalformedToken(
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
                #[diag(label = "the problematic term is here")]
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
    let span = SimpleSpan::new(
        "file://test.py",
        "print(1 + 2)\nprint(1 + '1')\nprint('1' + '1')",
        23,
        26,
    );
    let error = MyError::TypeError {
        term: "'1'".to_owned(),
        expected_ty: "int".to_owned(),
        actual_ty: "str".to_owned(),
        span: span.clone(),
    };

    assert_eq!(
        error.to_string(),
        "`'1'` is expected to be of type `int`, but is of type `str`."
    );
    assert_eq!(error.code(), "E00");
    assert_eq!(error.primary_span(), span);

    #[cfg(feature = "annotate-snippets")]
    eprintln!(
        "---------- annotate-snippets ----------\n{}",
        error
            .fmt_as_annotate_snippets_with_opts(annotate_snippets::display_list::FormatOptions {
                color: true,
                anonymized_line_numbers: false,
                margin: None,
            })
            .unwrap()
    );

    #[cfg(feature = "ariadne")]
    eprintln!(
        "---------- ariadne ----------\n{}",
        error.fmt_as_ariadne_report().unwrap()
    );

    #[cfg(feature = "miette")]
    eprintln!(
        "---------- miette (Narratable) ----------\n{}",
        error.fmt_as_miette_diagnostic_with(&miette::NarratableReportHandler::new())
    );
    #[cfg(feature = "miette")]
    eprintln!(
        "---------- miette (JSON) ----------\n{}",
        error.fmt_as_miette_diagnostic_with(&miette::JSONReportHandler::new())
    );
    #[cfg(feature = "miette")]
    eprintln!(
        "---------- miette (ASCII Graphical) ----------\n{}",
        error.fmt_as_miette_diagnostic_with(&miette::GraphicalReportHandler::new_themed(
            miette::GraphicalTheme::ascii()
        ))
    );
    #[cfg(feature = "miette")]
    eprintln!(
        "---------- miette (Unicode Graphical) ----------\n{}",
        error.fmt_as_miette_diagnostic_with(&miette::GraphicalReportHandler::new_themed(
            miette::GraphicalTheme::unicode()
        ))
    );
}
