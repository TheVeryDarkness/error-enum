//! Simple tests for error messages.
use error_enum::{ErrorEnum, SimpleSpan};
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
                    white: String,
                    /// Span
                    #[diag(span)]
                    span: SimpleSpan,
                },
            },
        },
    }
}

#[test]
fn basic() {
    let error = ColoredError::RedError(1, 2);

    assert_eq!(error.code(), "E01");
}

#[test]
#[cfg(feature = "ariadne")]
fn ariadne() {
    use ariadne::Config;
    let error = ColoredError::RedError(1, 2);

    {
        let s = error
            .fmt_as_ariadne_report_with(Config::new().with_color(false))
            .unwrap();
        assert_eq!(
            s,
            "\
[E01] Error: 1 and 2 is not red.
   ╭─[ :1:1 ]
   │
 1 │ 
───╯
"
        );
    }

    let error = ColoredError::WhiteError {
        white: "white".into(),
        span: SimpleSpan::new("foo.rs", "use white;", 4, 9),
    };
    {
        let s = error
            .fmt_as_ariadne_report_with(Config::new().with_color(false))
            .unwrap();
        assert_eq!(
            s,
            "\
[E05] Error: All in white.
   ╭─[ foo.rs:1:5 ]
   │
 1 │ use white;
───╯
"
        );
    }
}

#[test]
#[cfg(feature = "miette")]
fn miette() {
    use miette::{GraphicalReportHandler, GraphicalTheme, NarratableReportHandler};

    let error = ColoredError::RedError(1, 2);

    {
        let s = error.fmt_as_miette_diagnostic_with(&NarratableReportHandler::new());
        assert_eq!(
            s,
            "\
1 and 2 is not red.
    Diagnostic severity: error
Begin snippet for  starting at line 1, column 1

diagnostic code: E01
For more details, see:

"
        );
    }

    {
        let s = error.fmt_as_miette_diagnostic_with(&GraphicalReportHandler::new_themed(
            GraphicalTheme::none(),
        ));
        assert_eq!(
            s,
            "\
\u{1b}]8;;\u{1b}\\E01 (link)\u{1b}]8;;\u{1b}\\

  x 1 and 2 is not red.
   ,-[:1:1]
   `----
",
            "{s}"
        );
    }

    let error = ColoredError::WhiteError {
        white: "white".into(),
        span: SimpleSpan::new("foo.rs", "use white;", 4, 9),
    };
    {
        let s = error.fmt_as_miette_diagnostic_with(&GraphicalReportHandler::new_themed(
            GraphicalTheme::unicode_nocolor(),
        ));
        assert_eq!(
            s,
            "\
\u{1b}]8;;foo.rs\u{1b}\\E05 (link)\u{1b}]8;;\u{1b}\\

  × All in white.
   ╭─[foo.rs:1:5]
 1 │ use white;
   ·     ─────
   ╰────
"
        );
    }
}
