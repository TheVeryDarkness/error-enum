//! Integration tests for label source grouping in backends.

#![expect(clippy::panic)]
#![allow(clippy::unwrap_used)]

use core::fmt;
use error_enum_core::{vec1, ErrorType, ErrorTypeExt, Kind, LabelVec1, SimpleSpan};

#[derive(Debug)]
struct MultiLabelError {
    labels: LabelVec1<SimpleSpan, String>,
}

impl core::error::Error for MultiLabelError {}

impl fmt::Display for MultiLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multi label")
    }
}

impl ErrorType for MultiLabelError {
    type Span = SimpleSpan;
    type Message = String;
    type Label = String;

    fn kind(&self) -> Kind {
        Kind::Error
    }
    fn number(&self) -> &str {
        "0"
    }
    fn code(&self) -> &str {
        "E0"
    }
    fn primary_span(&self) -> Option<Self::Span> {
        Some(self.labels.first().0.clone())
    }
    fn primary_message(&self) -> Self::Message {
        "multi label".into()
    }
    fn primary_labels(&self) -> LabelVec1<Self::Span, Self::Label> {
        self.labels.clone()
    }
    fn additional(&self) -> error_enum_core::IterAdditional<Self> {
        Box::new([].into_iter())
    }
}

#[test]
#[cfg(feature = "annotate-snippets")]
fn annotate_snippets_merges_same_source_labels() {
    use annotate_snippets::display_list::FormatOptions;

    let base = SimpleSpan::new("file.rs", "one two", 0, 3);
    let labels: LabelVec1<SimpleSpan, String> = vec1![
        (base.clone(), "error here".into()),
        (base.with_range(4, 7), "also here".into()),
    ];
    let error = MultiLabelError { labels };
    let output = error.fmt_as_annotate_snippets_with_opts(FormatOptions::default());
    assert!(
        output.matches("--> file.rs").count() == 1,
        "expected one slice for shared source, got:\n{output}"
    );
    assert!(output.contains("error here"));
    assert!(output.contains("also here"));
}

#[test]
#[cfg(feature = "annotate-snippets")]
fn annotate_snippets_orders_sources_by_declaration() {
    use annotate_snippets::display_list::FormatOptions;

    let first = SimpleSpan::new("first.rs", "aaa", 0, 1);
    let second = SimpleSpan::new("second.rs", "bbb", 0, 1);
    let labels: LabelVec1<SimpleSpan, String> =
        vec1![(first, "on first".into()), (second, "on second".into()),];
    let error = MultiLabelError { labels };
    let output = error.fmt_as_annotate_snippets_with_opts(FormatOptions::default());
    let first_pos = output.find("first.rs").expect("first.rs slice");
    let second_pos = output.find("second.rs").expect("second.rs slice");
    assert!(
        first_pos < second_pos,
        "expected first.rs before second.rs:\n{output}"
    );
}
