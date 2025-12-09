use crate::{ErrorEnum, Kind, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::{Error, SimpleFiles},
    term::{termcolor::Buffer, Config, Styles, StylesWriter},
};
use std::io;

impl From<Kind> for Severity {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Error => Severity::Error,
            Kind::Warn => Severity::Warning,
        }
    }
}

pub(crate) type Files<T> =
    SimpleFiles<<<T as ErrorEnum>::Span as Span>::Uri, <<T as ErrorEnum>::Span as Span>::Source>;

pub(crate) fn to_codespan_diagnostic<T: ErrorEnum + ?Sized>(
    value: &T,
) -> (Diagnostic<usize>, Files<T>) {
    let diagnostic = Diagnostic {
        severity: value.kind().into(),
        code: Some(value.code().into()),
        message: value.primary_message().to_string(),
        labels: [
            Label::new(LabelStyle::Primary, 0, value.primary_span().range())
                .with_message(value.primary_label()),
        ]
        .into(),
        notes: Vec::new(),
    };

    // FIXME: implement my own `Files` to avoid cloning source texts and indexes
    let mut files = SimpleFiles::new();
    let primary_span = value.primary_span();
    files.add(
        primary_span.uri().clone(),
        primary_span.source_text().clone(),
    );

    (diagnostic, files)
}

pub(crate) fn fmt_as_codespan_diagnostic<T: ErrorEnum + ?Sized>(
    value: &T,
    config: Config,
    styles: Option<&Styles>,
) -> Result<String, Error> {
    let (diagnostic, files) = to_codespan_diagnostic(value);

    if let Some(styles) = styles {
        let mut buf = Buffer::ansi();
        let mut writer = StylesWriter::new(&mut buf, styles);
        codespan_reporting::term::emit_to_write_style(&mut writer, &config, &files, &diagnostic)?;

        String::from_utf8(buf.into_inner())
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::InvalidData, e)))
    } else {
        let mut buf = String::new();
        codespan_reporting::term::emit_to_string(&mut buf, &config, &files, &diagnostic)?;

        Ok(buf)
    }
}
