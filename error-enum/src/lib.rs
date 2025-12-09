#![doc = include_str!("../../readme.md")]
//! ## Syntax by Examples
//!
//! An example for `error_type`:
//!
//! ```rust
#![doc = include_str!("../examples/python.rs")]
//! ```

pub use error_enum_core::{ErrorEnum, Kind, SimpleSpan, Span};
#[cfg(feature = "derive")]
pub use error_enum_macros::{error_type, ErrorType};
