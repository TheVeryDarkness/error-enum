# Primary Diagnostic Attributes (Variant / Prefix)

| Attribute                                 | Description                                                                |
| ----------------------------------------- | -------------------------------------------------------------------------- |
| `#[diag(kind   = $kind:lit_str)]`         | Built-in only: `$kind` is `"error"` or `"warn"`. Default is `"error"`.     |
| `#[diag(kind   = $kind:expr)]`            | Any expression of the configured [`kind_type`](#custom-diagnostickind) (e.g. `MyKind::Bug`). |
| `#[diag(kind_type = $ty:lit_str)]`        | Override `ErrorType::Kind` (default `error_enum::Kind`). Must implement `DiagnosticKind`. |
| `#[diag(number = $number:lit_int)]`       | `$number` is the error number suffix.                                      |
| `#[diag(msg    = $msg:lit_str)]`          | `$msg` is the error message.                                               |
| `#[diag(label  = $label:lit_str)]`        | `$label` is the primary span label.                                        |
| `#[diag(span_type = $span_type:lit_str)]` | `$span_type` is the type of the span. Default is `error_enum::SimpleSpan`. |
| `#[diag(nested)]`                         | Mark this variant as a nested error wrapper.                               |

String `kind = "..."` is invalid when `kind_type` is set; use an expression instead.

## Custom `DiagnosticKind`

Implement `error_enum::DiagnosticKind` for your kind type:

- `code_prefix(&self) -> &str` — used by the default `ErrorType::code()` (`prefix` + `number`)
- Feature-gated `as_annotate_snippets` / `as_ariadne` / `as_codespan` / `as_miette` — **required** for each enabled backend; this crate does not auto-map custom kinds

```ignore
#[derive(Clone, Copy, Default)]
enum MyKind {
    #[default]
    Bug,
    Lint,
}

impl error_enum::DiagnosticKind for MyKind {
    fn code_prefix(&self) -> &str {
        match self {
            MyKind::Bug => "B",
            MyKind::Lint => "L",
        }
    }
    // ... implement as_* for enabled features ...
}

#[derive(Debug, error_enum::ErrorType)]
#[diag(kind_type = "MyKind")]
#[diag(kind = MyKind::Bug)]
#[diag(number = "01")]
#[diag(msg = "internal failure")]
struct Ice;
```

# Primary Span (Field)

| Attribute       | Description                                                |
| --------------- | ---------------------------------------------------------- |
| `#[diag(span)]` | Mark this field as the primary span of this error variant. |

# Subdiagnostic Attributes (Variant or Field)

Each subdiagnostic is a separate attribute. Use list syntax with a positional message string.

| Attribute                                     | Description                                                                                       |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `#[diag(note($msg:lit_str))]`                 | Additional note. On a field, attaches to that field's span.                                       |
| `#[diag(help($msg:lit_str))]`                 | Additional help. On a field, attaches to that field's span.                                       |
| `#[diag(label($label:lit_str))]`              | Secondary span label on a field; merged into primary labels or the same-field subdiagnostic unit. |
| `#[diag(note($msg:lit_str, label = $label))]` | Note with an optional span label override (field-level only).                                     |
| `#[diag(help($msg:lit_str, label = $label))]` | Help with an optional span label override (field-level only).                                     |

## `LabelVec1` convention

Each diagnostic unit (primary or additional) exposes labels as `LabelVec1<(Span, Label)>` — a non-empty `mitsein::vec1::Vec1`:

- Index `[0]` is the unit's primary label (the main span label for that unit).
- Further entries are secondary labels on the same unit, kept in **attribute declaration order** (variant attrs → field order → attribute order on a field).

Spanless notes/helps (variant-level or without `#[diag(span)]`) use `vec1![(Span::default(), message)]`; backends render the message as a footer when supported (see [Note vs help](#note-vs-help-backend-rendering)).

On a field, `note`, `help`, and `label("...")` on the **same field** are merged into one additional unit whose `LabelVec1` lists every label for that anchor span.

Additional units must not mix labels from different fields or spans. Put cross-span labels in primary `labels` or separate additional units.

## Source text grouping (backend rendering)

The macro and `ErrorType` API **keep every** `(Span, Label)` entry in declaration order. Backends group labels that [`share_source_text`](https://docs.rs/error-enum-core/latest/error_enum_core/trait.Span.html#method.share_source_text) into one underlying slice/file with multiple annotations. Groups for different sources appear in the order each source **first** shows up in the combined label list.

Example: primary labels `(span_a, "error")` and `(span_b, "also")` on `file.rs`, then `(span_c, "other")` on `other.rs` → one `file.rs` slice with two annotations, then one `other.rs` slice.

## Note vs help (backend rendering)

Messages from `note("...")` and `help("...")` are passed through as-is; this crate never adds `note:` or `help:` prefixes.

| Backend            | Note                        | Help                                    |
| ------------------ | --------------------------- | --------------------------------------- |
| annotate-snippets  | native `Note` annotation    | native `Help` annotation                |
| ariadne            | `with_note`                 | `with_help`                             |
| codespan-reporting | `Diagnostic::notes`         | same channel as note (no separate help) |
| miette             | spanless notes not rendered | `Diagnostic::help`                      |

## Migration

| Old syntax                     | New syntax                           |
| ------------------------------ | ------------------------------------ |
| `#[diag(note = "...")]`        | `#[diag(note("..."))]`               |
| `#[diag(help = "...")]`        | `#[diag(help("..."))]`               |
| field `#[diag(label = "...")]` | `#[diag(label("..."))]`              |
| `primary_label()`              | `primary_labels(): LabelVec1`        |
| `additional()` 4-tuple         | `(Message, LabelVec1, Note\|Help)`   |
| `AdditionalKind::Label`        | removed (labels live in `LabelVec1`) |
| `fn kind(&self) -> Kind`       | `type Kind: DiagnosticKind` + `fn kind(&self) -> Self::Kind` |
| `fn code(&self) -> &str`       | `fn code(&self) -> String` (default: `code_prefix` + `number`) |

Primary labels on variants still use `#[diag(label = "...")]`.
