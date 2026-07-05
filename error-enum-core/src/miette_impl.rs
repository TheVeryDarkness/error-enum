use crate::{AdditionalKind, ErrorType, Indexer, Kind, Span};
use alloc::{
    boxed::Box,
    string::{String, ToString as _},
    vec::Vec,
};
use core::{error::Error, fmt};
use miette::{
    Diagnostic, LabeledSpan, MietteError, MietteSpanContents, ReportHandler, Severity, SourceCode,
    SourceSpan, SpanContents,
};

pub(crate) struct Wrapper<'a, T: ?Sized, S>(&'a T, SpanWrapper<S>);

impl<'a, T: ErrorType<Span = S> + ?Sized, S: Span + Default> Wrapper<'a, T, S> {
    pub(crate) fn new(value: &'a T) -> Self {
        Self(value, SpanWrapper(value.primary_span().unwrap_or_default()))
    }
}

impl<T: ErrorType + 'static, S: Span + Send + Sync> Wrapper<'_, T, S> {
    pub(crate) fn fmt_with(&self, handler: &impl ReportHandler) -> String {
        WrapperWithHandler(self, handler).to_string()
    }
}

impl<T: ErrorType + ?Sized, S> fmt::Debug for Wrapper<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.primary_message())
    }
}
impl<T: ErrorType + ?Sized, S> fmt::Display for Wrapper<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.primary_message())
    }
}
impl<T: ErrorType + ?Sized, S> Error for Wrapper<'_, T, S> {}

fn is_placeholder_span<S: Span>(span: &S) -> bool {
    span.start() == span.end() && span.start() == 0 && span.uri().to_string().is_empty()
}

impl<T: ErrorType + ?Sized, S: Span + Send + Sync> Diagnostic for Wrapper<'_, T, S> {
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
        Some(Box::new(self.0.primary_span()?.uri().clone()))
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let mut labeled = Vec::new();
        let mut primary_index = 0usize;
        for (span, label) in self.0.primary_labels().iter() {
            if is_placeholder_span(span) {
                continue;
            }
            let labeled_span = if primary_index == 0 {
                LabeledSpan::new_primary_with_span(
                    Some(label.to_string()),
                    SourceSpan::new(span.start().into(), span.end() - span.start()),
                )
            } else {
                LabeledSpan::new_with_span(
                    Some(label.to_string()),
                    SourceSpan::new(span.start().into(), span.end() - span.start()),
                )
            };
            primary_index += 1;
            labeled.push(labeled_span);
        }
        for (message, labels, _kind) in self.0.additional() {
            let _ = message;
            for (span, label) in labels.iter() {
                if is_placeholder_span(span) {
                    continue;
                }
                labeled.push(LabeledSpan::new_with_span(
                    Some(label.to_string()),
                    SourceSpan::new(span.start().into(), span.end() - span.start()),
                ));
            }
        }
        Some(Box::new(labeled.into_iter()))
    }
    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        let mut parts = Vec::new();
        for (message, _labels, kind) in self.0.additional() {
            if !matches!(kind, AdditionalKind::Help) {
                continue;
            }
            let message = message.to_string();
            if !message.is_empty() {
                parts.push(message);
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(Box::new(parts.join("\n")))
        }
    }
}

struct WrapperWithHandler<'a, T, S, H: ?Sized>(&'a Wrapper<'a, T, S>, &'a H);

impl<T: ErrorType + 'static, S: Span + Send + Sync, H: ReportHandler + ?Sized> fmt::Display
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
        let index = self.0.source_index();
        let (start, end) = index.span_with_context_lines(
            span.offset(),
            span.offset() + span.len(),
            context_lines_before,
            context_lines_after,
        );
        let (start_line, start_column) = index.line_col_at(span.offset());
        let (end_line, _) = index.line_col_at(span.offset() + span.len());
        let name = self.0.uri().to_string();
        let data = &self.0.source_text().as_ref().as_bytes()[start..end];
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
