use crate::{Indexer, LineIndexer};
use alloc::sync::Arc;
use core::{fmt, ops::Range};

/// Trait for span types used in error enums.
pub trait Span: Clone {
    /// The URI type for the span.
    type Uri: PartialEq + Clone + fmt::Display;
    /// The source text type for the span.
    type Source: AsRef<str> + Clone + PartialEq;
    /// The index of the source text.
    type Index: Indexer + ?Sized;

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
    /// Check if the source text of the span is shared with another span.
    ///
    /// # Note
    ///
    /// This method may be insufficient for some use cases. For example, if `Uri` is `String`,
    /// this method will compare the string contents directly, which is `O(n)` time complexity.
    /// In such cases, please implement your own `share_source_text` method.
    fn share_source_text(&self, other: &Self) -> bool {
        self.uri() == other.uri() && self.source_text() == other.source_text()
    }
}

/// A simple implementation of [`Span`].
#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Returns a copy of this span with a different byte range, sharing source identity.
    pub fn with_range(&self, start: usize, end: usize) -> Self {
        Self {
            uri: self.uri.clone(),
            source: self.source.clone(),
            indexer: self.indexer.clone(),
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
