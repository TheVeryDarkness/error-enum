use crate::{ErrorEnum, Kind, Span};
use ariadne::{Config, Label, Report, ReportKind};
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
    <<T as ErrorEnum>::Span as Span>::Uri,
    ariadne::Source<<<T as ErrorEnum>::Span as Span>::Source>,
);

struct Cache<T: ErrorEnum + ?Sized> {
    sources: Vec<SourceEntry<T>>,
}

impl<T: ErrorEnum + ?Sized> FromIterator<T::Span> for Cache<T> {
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

impl<T: ErrorEnum + ?Sized> ariadne::Cache<<T::Span as Span>::Uri> for Cache<T> {
    type Storage = <T::Span as Span>::Source;

    fn fetch(
        &mut self,
        id: &<T::Span as Span>::Uri,
    ) -> Result<&ariadne::Source<Self::Storage>, impl std::fmt::Debug> {
        self.sources
            .iter()
            .find(|(uri, _)| uri == id)
            .map(|(_, source)| source)
            .ok_or("Source not found")
    }

    fn display<'a>(&self, id: &'a <T::Span as Span>::Uri) -> Option<impl std::fmt::Display + 'a> {
        self.sources
            .iter()
            .find(|(uri, _)| uri == id)
            .map(|(uri, _)| uri)
            .cloned()
    }
}

pub(crate) fn to_ariadne_report<T: ErrorEnum + ?Sized>(
    error: &T,
    buf: &mut impl io::Write,
    config: Config,
) -> Result<(), io::Error> {
    let primary_span = error.primary_span();
    let primary_message = error.primary_message();
    let cache: Cache<T> = Cache::from_iter(std::iter::once(primary_span.clone()));
    Report::build(error.kind().into(), SpanWrapper(primary_span.clone()))
        .with_code(error.code())
        .with_message(primary_message)
        .with_label(Label::new(SpanWrapper(primary_span)).with_message(error.primary_label()))
        .with_config(config)
        .finish()
        .write(cache, buf)
}
pub(crate) fn fmt_as_ariadne_report<T: ErrorEnum + ?Sized>(
    error: &T,
    config: Config,
) -> Result<String, io::Error> {
    let mut result = Vec::new();
    to_ariadne_report(error, &mut result, config)?;
    String::from_utf8(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
