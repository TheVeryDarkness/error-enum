//! Simple tests for error messages.
use error_enum::ErrorEnum;
use error_enum_macros::error_type;

error_type! {
    #[derive(Debug)]
    pub ColoredError {
        /// 测试
        {
            #[diag(kind = "error")]
            #[diag(number = "0")]
            #[diag(msg = "SimpleError")]
            {
                #[diag(number = "0")]
                #[diag(msg = "{0} is not black.")]
                BlackError (u8),
                #[diag(number = "1")]
                #[diag(msg = "{0} and {1} is not red.")]
                RedError (u8, u8),
                #[diag(number = "2")]
                #[diag(msg = "Code is green and yellow.")]
                GreenYellowError,
                #[diag(number = "3")]
                #[diag(msg = "I'm blue.")]
                BlueError,
                #[diag(number = "4")]
                #[diag(msg = "Purpule and cyan.")]
                PurpleCyanError,
                #[diag(number = "5")]
                #[diag(msg = "All in {white}.")]
                WhiteError {
                    /// Color name
                    white: String
                },
            },
        },
    }
}

#[test]
#[cfg(feature = "miette")]
fn miette() {
    use miette::{GraphicalReportHandler, GraphicalTheme, NarratableReportHandler};
    let report = ColoredError::RedError(1, 2).as_miette_diagnostic();

    {
        let mut s = String::new();
        NarratableReportHandler::new()
            .render_report(&mut s, &report)
            .unwrap();
        assert_eq!(
            s,
            "\
1 and 2 is not red.
    Diagnostic severity: error
diagnostic code: 01
"
        );
    }

    {
        let mut s = String::new();
        GraphicalReportHandler::new_themed(GraphicalTheme::ascii())
            .render_report(&mut s, &report)
            .unwrap();
    }
}
