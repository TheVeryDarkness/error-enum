# Error Enum

[![crates.io](https://img.shields.io/crates/v/error-enum.svg)](https://crates.io/crates/error-enum)
[![docs.rs](https://docs.rs/error-enum/badge.svg)](https://docs.rs/error-enum)
[![codecov](https://codecov.io/github/theverydarkness/error-enum/graph/badge.svg?token=70168G1POP)](https://codecov.io/github/theverydarkness/error-enum)

Used to generate documentation comments and `Display` implementation for tree-structured error types.

It also provides error rendering capabilities with colorful and detailed error messages, via implementing those traits or structs from crates listed below:

- `ariadne::Report` (if `ariadne` feature is enabled)
- `annotate_snippets::display_list::DisplayList` (if `annotate-snippets` feature is enabled)
- `codespan_reporting::diagnostic::Diagnostic` and `codespan_reporting::files::SimpleFiles` (if `codespan-reporting` feature is enabled)
- `miette::Diagnostic` (if `miette` feature is enabled)

## Concepts

|    Concept     |            Example             |
| :------------: | :----------------------------: |
|     Number     |             `1234`             |
|      Code      |            `E1234`             |
|      Kind      |            `error`             |
|  Kind Acronym  |              `E`               |
|      Kind      |         `error[E1234]`         |
| Message Prefix |        `error[E1234]: `        |
|  Description   |        `Access denied.`        |
|    Message     | `error[E1234]: Access denied.` |
