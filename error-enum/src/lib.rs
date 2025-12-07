#![doc = include_str!("../../readme.md")]
//! ## Syntax by Examples
//!
//! An example for `error_type`:
//!
//! ```rust
#![doc = include_str!("../examples/python.rs")]
//! ```

pub use error_enum_macros::{error_type, ErrorType};
use std::{fmt, ops::Range, rc::Rc, sync::Arc};
use stringzilla::sz::find_newline_utf8;

#[cfg(feature = "annotate-snippets")]
mod annotate_snippets_impl;
#[cfg(feature = "ariadne")]
mod ariadne_impl;
#[cfg(feature = "miette")]
mod miette_impl;

/// Enum representing the kind of an error.
#[derive(Clone, Copy, Default)]
pub enum Kind {
    /// Error kind.
    #[default]
    Error,
    /// Warning kind.
    Warn,
}

impl Kind {
    /// Get short representation of the [Kind].
    pub fn short_str(&self) -> &'static str {
        match self {
            Kind::Error => "E",
            Kind::Warn => "W",
        }
    }
}

/// A indexable string.
pub trait Indexer {
    /// Returns the line and column number of this `Position`.
    fn line_col_at(&self, pos: usize) -> (usize, usize);

    /// Returns the start and the end of the line that contains the position at `pos`.
    fn line_span_at(&self, pos: usize) -> (usize, usize);

    /// Returns the start and the end of the `(context_lines_before + 1 + context_lines_after)`
    /// lines that contains the position at `pos`.
    fn span_with_context_lines(
        &self,
        start: usize,
        end: usize,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> (usize, usize);
}

macro_rules! impl_indexable {
    ($T:ty) => {
        impl<T: Indexer + ?Sized> Indexer for $T {
            fn line_col_at(&self, pos: usize) -> (usize, usize) {
                T::line_col_at(self, pos)
            }

            fn line_span_at(&self, pos: usize) -> (usize, usize) {
                T::line_span_at(self, pos)
            }

            fn span_with_context_lines(
                &self,
                start: usize,
                end: usize,
                context_lines_before: usize,
                context_lines_after: usize,
            ) -> (usize, usize) {
                T::span_with_context_lines(
                    self,
                    start,
                    end,
                    context_lines_before,
                    context_lines_after,
                )
            }
        }
    };
}

impl_indexable!(Box<T>);
impl_indexable!(Rc<T>);
impl_indexable!(Arc<T>);

/// An [`Indexer`] that stores ending positions of every line (including trailing newlines).
#[derive(Debug)]
#[repr(transparent)]
pub struct LineIndexer([usize]);

impl LineIndexer {
    /// Create an [`LineIndexer`].
    pub fn new(s: &str) -> Box<Self> {
        let mut line_starts = Vec::new();
        let slice = s.as_bytes();
        while let Some(index) = find_newline_utf8(slice) {
            line_starts.push(index.end());
        }
        line_starts.push(s.len());
        let line_starts = line_starts.into_boxed_slice();
        unsafe { std::mem::transmute(line_starts) }
    }
}

impl LineIndexer {
    fn line_start_at(&self, pos: usize) -> usize {
        match self.0.binary_search(&pos) {
            Ok(i) => self.0[i],
            Err(0) => 0,
            Err(i) => self.0[i.saturating_sub(1)],
        }
    }
    fn line_at(&self, pos: usize) -> usize {
        match self.0.binary_search(&pos) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }
}

impl Indexer for LineIndexer {
    fn line_col_at(&self, pos: usize) -> (usize, usize) {
        let line_start = self.line_start_at(pos);
        debug_assert!(pos >= line_start);
        (line_start, pos - line_start)
    }

    fn line_span_at(&self, pos: usize) -> (usize, usize) {
        match self.0.binary_search(&pos) {
            Ok(i) if i + 1 == self.0.len() => (self.0[i], self.0[i]),
            Ok(i) => (self.0[i], self.0[i + 1]),
            Err(0) => (0, self.0[0]),
            Err(i) if i == self.0.len() => {
                let j = i.saturating_sub(1);
                (self.0[j], self.0[j])
            }
            Err(i) => (self.0[i.saturating_sub(1)], self.0[i]),
        }
    }

    fn span_with_context_lines(
        &self,
        start: usize,
        end: usize,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> (usize, usize) {
        let start = if context_lines_before == 0 {
            start
        } else {
            self.line_at(start).saturating_sub(context_lines_before)
        };
        let end = if context_lines_after == 0 {
            end
        } else {
            self.line_at(end)
                .saturating_add(context_lines_after)
                .min(self.0.len())
        };
        (start, end)
    }
}

/// Trait for span types used in error enums.
pub trait Span: Clone {
    /// The URI type for the span.
    type Uri: PartialEq + Clone + fmt::Display;
    /// The source text type for the span.
    type Source: AsRef<str> + Clone;
    /// The index of the source text.
    type Index: Indexer;

    /// Get the start position of the span.
    fn start(&self) -> usize;
    /// Get the end position of the span.
    fn end(&self) -> usize;
    /// Get the range of the span.
    fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }
    /// Get the source text of the span.
    fn source_text(&self) -> &Self::Source;
    /// Get the index of the source.
    fn source_index(&self) -> &Self::Index;
    /// Get the URI of the span.
    fn uri(&self) -> &Self::Uri;
}

/// A simple implementation of [`Span`].
#[derive(Clone, Debug)]
pub struct SimpleSpan {
    uri: Arc<str>,
    source: Arc<str>,
    indexer: Arc<LineIndexer>,
    start: usize,
    end: usize,
}

impl SimpleSpan {
    /// Create a new [`SimpleSpan`].
    pub fn new(
        uri: impl Into<Arc<str>>,
        source: impl Into<Arc<str>>,
        start: usize,
        end: usize,
    ) -> Self {
        let uri = uri.into();
        let source = source.into();
        let indexer = LineIndexer::new(&source).into();
        Self {
            uri,
            source,
            indexer,
            start,
            end,
        }
    }
}

impl Span for SimpleSpan {
    type Uri = Arc<str>;
    type Source = Arc<str>;
    type Index = Arc<LineIndexer>;

    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
    fn source_text(&self) -> &Self::Source {
        &self.source
    }
    fn source_index(&self) -> &Self::Index {
        &self.indexer
    }
    fn uri(&self) -> &Self::Uri {
        &self.uri
    }
}

impl Default for SimpleSpan {
    fn default() -> Self {
        Self::new("", "", 0, 0)
    }
}

impl From<&SimpleSpan> for SimpleSpan {
    fn from(value: &SimpleSpan) -> Self {
        value.clone()
    }
}

/// Trait for error enums generated by [`error_type!`] macro and [`ErrorType`] derive macro.
pub trait ErrorEnum: std::error::Error {
    /// The span type associated with the error enum.
    type Span: Span;
    /// The message type associated with the error enum.
    type Message: fmt::Display;

    /// Get the kind of the error.
    fn kind(&self) -> Kind;
    /// Get the number of the error.
    fn number(&self) -> &str;
    /// Get the code of the error.
    ///
    /// Normally the code is a combination of kind short string and number,
    /// like "E0", "W1", etc.
    fn code(&self) -> &str;
    /// Get the primary span and message of the error.
    fn primary_span(&self) -> Self::Span;
    /// Get the primary span and message of the error.
    fn primary_message(&self) -> Self::Message;

    /// Format the error as an [annotate snippet].
    ///
    /// [annotate snippet]: https://docs.rs/annotate-snippets/0.9.1/annotate_snippets/snippet/struct.Snippet.html
    #[cfg(feature = "annotate-snippets")]
    fn fmt_as_annotate_snippets(&self) -> Result<String, std::io::Error> {
        let result = annotate_snippets_impl::fmt_as_annotate_snippets(
            self,
            annotate_snippets::display_list::FormatOptions::default(),
        );
        Ok(result)
    }
    /// Format the error as an [annotate snippet] with [format options].
    ///
    /// [annotate snippet]: https://docs.rs/annotate-snippets/0.9.1/annotate_snippets/snippet/struct.Snippet.html
    /// [format options]: https://docs.rs/annotate-snippets/0.9.1/annotate_snippets/display_list/struct.FormatOptions.html
    #[cfg(feature = "annotate-snippets")]
    fn fmt_as_annotate_snippets_with_opts(
        &self,
        opts: annotate_snippets::display_list::FormatOptions,
    ) -> Result<String, std::io::Error> {
        let result = annotate_snippets_impl::fmt_as_annotate_snippets(self, opts);
        Ok(result)
    }

    /// Format the error as an [Ariadne report].
    ///
    /// [Ariadne report]: https://docs.rs/ariadne/0.6.0/ariadne/struct.Report.html
    #[cfg(feature = "ariadne")]
    fn fmt_as_ariadne_report(&self) -> Result<String, std::io::Error> {
        let mut result = Vec::new();
        ariadne_impl::to_ariadne_report(self, &mut result)?;
        Ok(String::from_utf8(result).unwrap())
    }

    /// Convert the error to a [Miette diagnostic].
    ///
    /// [Miette diagnostic]: https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html
    #[cfg(feature = "miette")]
    fn as_miette_diagnostic(&self) -> impl miette::Diagnostic + '_
    where
        Self::Span: Send + Sync,
    {
        miette_impl::Wrapper::new(self)
    }
    /// Format the error as a [Miette diagnostic] with a [Miette handler].
    ///
    /// [Miette diagnostic]: https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html
    /// [Miette Handler]: https://docs.rs/miette/7.6.0/miette/trait.ReportHandler.html
    #[cfg(feature = "miette")]
    fn fmt_as_miette_diagnostic_with(&self, handler: &impl miette::ReportHandler) -> String
    where
        Self: 'static + Sized,
        Self::Span: Send + Sync,
    {
        miette_impl::Wrapper::new(self).fmt_with(handler)
    }
}

impl<T: ErrorEnum + ?Sized> ErrorEnum for &T {
    type Span = T::Span;
    type Message = T::Message;

    fn kind(&self) -> Kind {
        (*self).kind()
    }
    fn number(&self) -> &str {
        (*self).number()
    }
    fn code(&self) -> &str {
        (*self).code()
    }
    fn primary_span(&self) -> Self::Span {
        (*self).primary_span()
    }
    fn primary_message(&self) -> Self::Message {
        (*self).primary_message()
    }

    #[cfg(feature = "annotate-snippets")]
    fn fmt_as_annotate_snippets(&self) -> Result<String, std::io::Error> {
        (*self).fmt_as_annotate_snippets()
    }
    #[cfg(feature = "annotate-snippets")]
    fn fmt_as_annotate_snippets_with_opts(
        &self,
        opts: annotate_snippets::display_list::FormatOptions,
    ) -> Result<String, std::io::Error> {
        (*self).fmt_as_annotate_snippets_with_opts(opts)
    }

    #[cfg(feature = "ariadne")]
    fn fmt_as_ariadne_report(&self) -> Result<String, std::io::Error> {
        (*self).fmt_as_ariadne_report()
    }

    #[cfg(feature = "miette")]
    fn as_miette_diagnostic(&self) -> impl miette::Diagnostic + '_
    where
        Self::Span: Send + Sync,
    {
        (*self).as_miette_diagnostic()
    }
}
