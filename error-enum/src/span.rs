use crate::{Indexer, LineIndexer};
use std::{fmt, ops::Range, sync::Arc};

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
