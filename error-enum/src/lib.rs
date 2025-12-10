//! # `error-enum`
//!
//! A Rust crate for defining error enums with rich diagnostics support.
//!
//! For features and concepts, please refer to the [readme](https://crates.io/crates/error-enum)
//! of this crate.
//!
//! ## Syntax by Examples
//!
//! An example for `error_type!`:
//!
//! ```rust
#![doc = include_str!("../examples/python.rs")]
//! ```
//!
//! Two examples for `ErrorType!`:
//!
//! ```rust
#![doc = include_str!("../examples/wrapper.rs")]
//! ```

pub use error_enum_core::{ErrorEnum, Kind, SimpleSpan, Span};
pub use error_enum_macros::{error_type, ErrorType};
