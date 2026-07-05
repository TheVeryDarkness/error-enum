use crate::{labels::group_labels_by_source, AdditionalKind, ErrorType, Kind, Span};
use alloc::{format, string::{String, ToString as _}, vec::Vec};
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
    SimpleFiles<<<T as ErrorType>::Span as Span>::Uri, <<T as ErrorType>::Span as Span>::Source>;

fn is_placeholder_span<S: Span>(span: &S) -> bool {
    span.start() == span.end() && span.start() == 0 && span.uri().to_string().is_empty()
}

pub(crate) fn to_codespan_diagnostic<T: ErrorType + ?Sized>(
    value: &T,
) -> (Diagnostic<usize>, Files<T>) {
    let mut files = SimpleFiles::new();
    let mut file_ids: Vec<(usize, T::Span)> = Vec::new();
    let mut resolve_file = |span: &T::Span| -> usize {
        for (id, existing) in &file_ids {
            if existing.share_source_text(span) {
                return *id;
            }
        }
        let id = files.add(span.uri().clone(), span.source_text().clone());
        file_ids.push((id, span.clone()));
        id
    };

    let primary_labels = value.primary_labels();
    let mut labels = Vec::new();
    let mut notes = Vec::new();
    let mut ordered: Vec<(usize, T::Span, String)> = Vec::new();
    let mut order = 0usize;
    for (span, label) in primary_labels.iter().cloned() {
        ordered.push((order, span, label.to_string()));
        order += 1;
    }
    for (message, unit_labels, kind) in value.additional() {
        let message = message.to_string();
        let mut has_real_span = false;
        for (span, label) in unit_labels.iter().cloned() {
            if is_placeholder_span(&span) {
                continue;
            }
            has_real_span = true;
            ordered.push((order, span, label.to_string()));
            order += 1;
        }
        if has_real_span {
            if !message.is_empty() && message != unit_labels.first().1.to_string() {
                notes.push(match kind {
                    AdditionalKind::Note => message,
                    AdditionalKind::Help => format!("help: {message}"),
                });
            }
        } else {
            notes.push(match kind {
                AdditionalKind::Note => message,
                AdditionalKind::Help => format!("help: {message}"),
            });
        }
    }
    let groups = group_labels_by_source(ordered);
    let mut label_index = 0usize;
    for group in groups {
        let file_id = resolve_file(&group.source);
        for (span, label) in group.entries {
            let is_primary = label_index == 0;
            label_index += 1;
            labels.push(
                Label::new(
                    if is_primary {
                        LabelStyle::Primary
                    } else {
                        LabelStyle::Secondary
                    },
                    file_id,
                    span.range(),
                )
                .with_message(label),
            );
        }
    }
    let diagnostic = Diagnostic {
        severity: value.kind().into(),
        code: Some(value.code().into()),
        message: value.primary_message().to_string(),
        labels: labels.into(),
        notes,
    };

    (diagnostic, files)
}

pub(crate) fn fmt_as_codespan_diagnostic<T: ErrorType + ?Sized>(
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
