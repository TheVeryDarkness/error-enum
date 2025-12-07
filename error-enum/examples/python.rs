//! An example.

use error_enum::{error_type, SimpleSpan};

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
        span: SimpleSpan::new("test.py", "1 + '1'", 4, 7),
    };

    assert_eq!(
        error.to_string(),
        "`'1'` is expected to be of type `int`, but is of type `str`."
    );
}
