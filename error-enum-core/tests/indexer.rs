//! Tests for the [`Indexer`] implementations.
use error_enum_core::Indexer;

#[test]
fn line_indexer_works() {
    use error_enum_core::LineIndexer;
    let text = "Hello\nWorld\nThis is a test.";
    let indexer = LineIndexer::new(text);

    assert_eq!(indexer.line_col_at(0), (0, 0)); // 'H'
    assert_eq!(indexer.line_col_at(3), (0, 3)); // 'l'
    assert_eq!(indexer.line_col_at(6), (1, 0)); // 'W'
    assert_eq!(indexer.line_col_at(11), (1, 5)); // 'd'
    assert_eq!(indexer.line_col_at(12), (2, 0)); // 'T'
    assert_eq!(indexer.line_col_at(21), (2, 9)); // 't'
    assert_eq!(indexer.line_col_at(26), (2, 14)); // '.'
    assert_eq!(indexer.line_col_at(27), (3, 0)); // EOF
    assert_eq!(indexer.line_col_at(30), (3, 3)); // beyond EOF

    assert_eq!(indexer.line_span_at(0), (0, 6)); // 'Hello\n'
    assert_eq!(indexer.line_span_at(3), (0, 6)); // 'Hello\n'
    assert_eq!(indexer.line_span_at(6), (6, 12)); // 'World\n'
    assert_eq!(indexer.line_span_at(11), (6, 12)); // 'World\n'
    assert_eq!(indexer.line_span_at(12), (12, 27)); // 'This is a test.'
    assert_eq!(indexer.line_span_at(21), (12, 27)); // 'This is a test.'
    assert_eq!(indexer.line_span_at(26), (12, 27)); // 'This is a test.'
    assert_eq!(indexer.line_span_at(27), (27, 27)); // EOF
    assert_eq!(indexer.line_span_at(30), (27, 27)); // beyond EOF

    assert_eq!(indexer.span_with_context_lines(7, 11, 0, 0), (6, 12)); // 'Hello\nWorld'
    assert_eq!(indexer.span_with_context_lines(7, 11, 1, 0), (0, 12)); // 'Hello\nWorld'
    assert_eq!(indexer.span_with_context_lines(7, 11, 2, 2), (0, 27)); // entire text
    assert_eq!(indexer.span_with_context_lines(0, 5, 1, 1), (0, 12)); // 'Hello\nWorld\n'
    assert_eq!(indexer.span_with_context_lines(0, 5, 2, 2), (0, 27)); // entire text
    assert_eq!(indexer.span_with_context_lines(22, 26, 1, 1), (6, 27)); // 'World\nThis is a test.'
    assert_eq!(indexer.span_with_context_lines(22, 26, 2, 2), (0, 27)); // entire text
}
