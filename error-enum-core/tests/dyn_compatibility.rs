//! Tests for the [`Indexer`] implementations.
use error_enum_core::{ErrorType, Kind, SimpleSpan};

#[expect(dead_code)]
fn dyn_compatibility(
    _: Box<dyn ErrorType<Span = SimpleSpan, Kind = Kind, Message = String, Label = String>>,
) {
}
