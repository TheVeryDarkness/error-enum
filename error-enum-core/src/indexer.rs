use std::{rc::Rc, sync::Arc};
use stringzilla::sz::find_newline_utf8;

/// A indexable string.
pub trait Indexer {
    /// Returns the line and column number of this `Position`.
    fn line_col_at(&self, pos: usize) -> (usize, usize);

    /// Returns the start and the end of the line that contains the position at `pos`.
    fn line_span_at(&self, pos: usize) -> (usize, usize);

    /// Returns the start and the end of the `(context_lines_before + n + context_lines_after)`
    /// lines that contains the span from `start` to `end`.
    ///
    /// `context_lines_before` and `context_lines_after` specify how many lines before and after
    /// the span to include.
    ///
    /// If there are not enough lines before or after, it will include as many as possible.
    /// And if `context_lines_before` or `context_lines_after` is zero, no extra lines will be included.
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
///
/// The line and column numbers are zero-based.
///
/// And note that the `LineIndexer` works as if there is an implicit newline at the end of the text.
#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct LineIndexer([usize]);

impl LineIndexer {
    /// Create an [`LineIndexer`].
    pub fn new(s: &str) -> Box<Self> {
        let mut line_starts = Vec::new();
        let mut cur = 0usize;
        let mut slice = s.as_bytes();
        while let Some(index) = find_newline_utf8(slice) {
            line_starts.push(cur + index.end());
            cur += index.end();
            slice = &slice[index.end()..]
        }
        line_starts.push(s.len());
        let line_starts = line_starts.into_boxed_slice();
        unsafe { std::mem::transmute(line_starts) }
    }
}

impl LineIndexer {
    /// Get the line number and the starting position of the line at `pos`.
    fn line_start_at(&self, pos: usize) -> usize {
        match self.0.binary_search(&pos) {
            Ok(i) => self.0[i],
            Err(0) => 0,
            Err(i) => self.0[i.saturating_sub(1)],
        }
    }
    /// Get the line number and the starting position of the line at `pos`.
    fn line_and_start_at(&self, pos: usize) -> (usize, usize) {
        match self.0.binary_search(&pos) {
            Ok(i) => (i + 1, self.0[i]),
            Err(0) => (0, 0),
            Err(i) => (i, self.0[i.saturating_sub(1)]),
        }
    }
    /// Get the line number at `pos`.
    fn line_at(&self, pos: usize) -> usize {
        match self.0.binary_search(&pos) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }
}

impl Indexer for LineIndexer {
    fn line_col_at(&self, pos: usize) -> (usize, usize) {
        let (line, line_start) = self.line_and_start_at(pos);
        debug_assert!(pos >= line_start);
        (line, pos - line_start)
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
            self.line_start_at(start)
        } else {
            self.line_at(start)
                .saturating_sub(context_lines_before)
                .checked_sub(1)
                .map_or_else(|| 0, |i| self.0[i])
        };
        let end = if context_lines_after == 0 {
            self.line_span_at(end).1
        } else {
            self.0[self
                .line_at(end)
                .saturating_add(context_lines_after)
                .min(self.0.len() - 1)]
        };
        (start, end)
    }
}
