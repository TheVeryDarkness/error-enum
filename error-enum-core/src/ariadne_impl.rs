use crate::{AdditionalKind, ErrorType, Kind, Span};
use alloc::{string::{String, ToString as _}, vec::Vec};
use ariadne::{Config, Label, Report, ReportKind};
use core::fmt;
use std::io;

impl From<Kind> for ReportKind<'_> {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Error => ReportKind::Error,
            Kind::Warn => ReportKind::Warning,
        }
    }
}

pub(crate) struct SpanWrapper<T>(T);

impl<T: Span> ariadne::Span for SpanWrapper<T> {
    type SourceId = T::Uri;

    fn source(&self) -> &Self::SourceId {
        self.0.uri()
    }
    fn start(&self) -> usize {
        Span::start(&self.0)
    }
    fn end(&self) -> usize {
        Span::end(&self.0)
    }
}

type SourceEntry<T> = (
    <<T as ErrorType>::Span as Span>::Uri,
    ariadne::Source<<<T as ErrorType>::Span as Span>::Source>,
);

struct Cache<T: ErrorType + ?Sized> {
    sources: Vec<SourceEntry<T>>,
}

impl<T: ErrorType + ?Sized> FromIterator<T::Span> for Cache<T> {
    fn from_iter<I: IntoIterator<Item = T::Span>>(iter: I) -> Self {
        let sources = iter
            .into_iter()
            .map(
                |span| -> (
                    <T::Span as Span>::Uri,
                    ariadne::Source<<T::Span as Span>::Source>,
                ) {
                    (
                        span.uri().clone(),
                        ariadne::Source::from(span.source_text().clone()),
                    )
                },
            )
            .collect();
        Self { sources }
    }
}

impl<T: ErrorType + ?Sized> ariadne::Cache<<T::Span as Span>::Uri> for Cache<T> {
    type Storage = <T::Span as Span>::Source;

    fn fetch(
        &mut self,
        id: &<T::Span as Span>::Uri,
    ) -> Result<&ariadne::Source<Self::Storage>, impl fmt::Debug> {
        self.sources
            .iter()
            .find(|(uri, _)| uri == id)
            .map(|(_, source)| source)
            .ok_or("Source not found")
    }

    fn display<'a>(&self, id: &'a <T::Span as Span>::Uri) -> Option<impl fmt::Display + 'a> {
        self.sources
            .iter()
            .find(|(uri, _)| uri == id)
            .map(|(uri, _)| uri)
            .cloned()
    }
}

fn is_placeholder_span<S: Span>(span: &S) -> bool {
    span.start() == span.end() && span.start() == 0 && span.uri().to_string().is_empty()
}

pub(crate) fn to_ariadne_report<T: ErrorType + ?Sized>(
    error: &T,
    buf: &mut impl io::Write,
    config: Config,
) -> Result<(), io::Error> {
    let primary_labels = error.primary_labels();
    let primary_span = primary_labels.first().0.clone();
    let mut spans: Vec<T::Span> = Vec::new();
    for (span, _) in primary_labels.iter().cloned() {
        spans.push(span);
    }
    for (message, labels, kind) in error.additional() {
        let _ = (message, kind);
        for (span, _) in labels.iter().cloned() {
            if !is_placeholder_span(&span) {
                spans.push(span);
            }
        }
    }
    let cache: Cache<T> = Cache::from_iter(spans);
    let mut builder = Report::build(error.kind().into(), SpanWrapper(primary_span.clone()))
        .with_code(error.code())
        .with_message(error.primary_message())
        .with_config(config);
    for (span, label) in primary_labels.iter().cloned() {
        builder = builder.with_label(Label::new(SpanWrapper(span)).with_message(label));
    }
    for (message, labels, kind) in error.additional() {
        match kind {
            AdditionalKind::Note if labels.iter().all(|(span, _)| is_placeholder_span(span)) => {
                builder = builder.with_note(message);
            }
            AdditionalKind::Help if labels.iter().all(|(span, _)| is_placeholder_span(span)) => {
                builder = builder.with_help(message);
            }
            AdditionalKind::Note | AdditionalKind::Help => {
                for (span, label) in labels.iter().cloned() {
                    if is_placeholder_span(&span) {
                        continue;
                    }
                    builder = builder.with_label(Label::new(SpanWrapper(span)).with_message(label));
                }
                if !message.to_string().is_empty()
                    && message.to_string() != labels.first().1.to_string()
                {
                    builder = match kind {
                        AdditionalKind::Note => builder.with_note(message),
                        AdditionalKind::Help => builder.with_help(message),
                    };
                }
            }
        }
    }
    builder.finish().write(cache, buf)
}
pub(crate) fn fmt_as_ariadne_report<T: ErrorType + ?Sized>(
    error: &T,
    config: Config,
) -> Result<String, io::Error> {
    let mut result = Vec::new();
    to_ariadne_report(error, &mut result, config)?;
    String::from_utf8(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
