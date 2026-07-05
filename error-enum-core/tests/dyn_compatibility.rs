//! Tests for the [`Indexer`] implementations.
use error_enum_core::{ErrorType, SimpleSpan};

#[expect(dead_code)]
fn dyn_compatibility(_: Box<dyn ErrorType<Span = SimpleSpan, Message = String, Label = String>>) {}
