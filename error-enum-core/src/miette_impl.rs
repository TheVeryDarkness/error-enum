use crate::{ErrorEnum, Indexer, Kind, Span};
use miette::{
    Diagnostic, LabeledSpan, MietteError, MietteSpanContents, ReportHandler, Severity, SourceCode,
    SourceSpan, SpanContents,
};
use std::{error::Error, fmt};

pub(crate) struct Wrapper<'a, T: ?Sized, S>(&'a T, SpanWrapper<S>);

impl<'a, T: ErrorEnum<Span = S> + ?Sized, S: Span> Wrapper<'a, T, S> {
    pub(crate) fn new(value: &'a T) -> Self {
        Self(value, SpanWrapper(value.primary_span()))
    }
}

impl<T: ErrorEnum + 'static, S: Span + Send + Sync> Wrapper<'_, T, S> {
    pub(crate) fn fmt_with(&self, handler: &impl ReportHandler) -> String {
        WrapperWithHandler(self, handler).to_string()
    }
}

impl<T: ErrorEnum + ?Sized, S> fmt::Debug for Wrapper<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.primary_message())
    }
}
impl<T: ErrorEnum + ?Sized, S> fmt::Display for Wrapper<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.primary_message())
    }
}
impl<T: ErrorEnum + ?Sized, S> Error for Wrapper<'_, T, S> {}

impl<T: ErrorEnum + ?Sized, S: Span + Send + Sync> Diagnostic for Wrapper<'_, T, S> {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.0.code()))
    }
    fn severity(&self) -> Option<Severity> {
        match self.0.kind() {
            Kind::Error => Some(Severity::Error),
            Kind::Warn => Some(Severity::Warning),
        }
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.1)
    }
    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.0.primary_span().uri().clone()))
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let primary_span = self.0.primary_span();
        let iter = [LabeledSpan::new_primary_with_span(
            Some(self.0.primary_label().to_string()),
            SourceSpan::new(
                primary_span.start().into(),
                primary_span.end() - primary_span.start(),
            ),
        )]
        .into_iter();
        Some(Box::new(iter))
    }
}

struct WrapperWithHandler<'a, T, S, H: ?Sized>(&'a Wrapper<'a, T, S>, &'a H);

impl<T: ErrorEnum + 'static, S: Span + Send + Sync, H: ReportHandler + ?Sized> fmt::Display
    for WrapperWithHandler<'_, T, S, H>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.debug(self.0, f)
    }
}

struct SpanWrapper<S>(S);

impl<S: Span + Send + Sync> SourceCode for SpanWrapper<S> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        // dbg!(span, context_lines_before, context_lines_after);
        // debug_assert!(
        //     span.offset() + span.len() < self.0.source_text().as_ref().len(),
        //     "{} + {} < {} does not hold",
        //     span.offset(),
        //     span.len(),
        //     self.0.source_text().as_ref().len(),
        // );

        let index = self.0.source_index();
        let (start, end) = index.span_with_context_lines(
            span.offset(),
            span.offset() + span.len(),
            context_lines_before,
            context_lines_after,
        );
        let (start_line, start_column) = index.line_col_at(start);
        let (end_line, _) = index.line_col_at(start);
        // dbg!(start, end, start_line, start_column, end_line);
        let name = self.0.uri().to_string();
        let data = &self.0.source_text().as_ref().as_bytes()[start..end];
        // dbg!(&name, data);
        Ok(Box::new(MietteSpanContents::new_named(
            name,
            data,
            SourceSpan::new(start.into(), end - start),
            start_line,
            start_column,
            end_line - start_line + 1,
        )))
    }
}
