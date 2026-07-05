# Primary Diagnostic Attributes (Variant / Prefix)

| Attribute                                 | Description                                                                |
| ----------------------------------------- | -------------------------------------------------------------------------- |
| `#[diag(kind   = $kind:lit_str)]`         | `$kind` is either `"error"` or `"warn"`. Default is `"error"`.             |
| `#[diag(number = $number:lit_int)]`       | `$number` is the error number suffix.                                      |
| `#[diag(msg    = $msg:lit_str)]`          | `$msg` is the error message.                                               |
| `#[diag(label  = $label:lit_str)]`         | `$label` is the primary span label.                                        |
| `#[diag(span_type = $span_type:lit_str)]` | `$span_type` is the type of the span. Default is `error_enum::SimpleSpan`. |
| `#[diag(nested)]`                         | Mark this variant as a nested error wrapper.                               |

# Primary Span (Field)

| Attribute       | Description                                                |
| --------------- | ---------------------------------------------------------- |
| `#[diag(span)]` | Mark this field as the primary span of this error variant. |

# Subdiagnostic Attributes (Variant or Field)

Each subdiagnostic is a separate attribute. Use list syntax with a positional message string.

| Attribute                                      | Description                                                                 |
| ---------------------------------------------- | --------------------------------------------------------------------------- |
| `#[diag(note($msg:lit_str))]`                  | Additional note. On a field, attaches to that field's span.                 |
| `#[diag(help($msg:lit_str))]`                  | Additional help. On a field, attaches to that field's span.                 |
| `#[diag(label($label:lit_str))]`               | Secondary span label. Only valid on fields.                                 |
| `#[diag(note($msg:lit_str, label = $label))]`  | Note with an optional span label override (field-level only).               |
| `#[diag(help($msg:lit_str, label = $label))]`  | Help with an optional span label override (field-level only).               |

## Migration

| Old syntax                    | New syntax                 |
| ----------------------------- | -------------------------- |
| `#[diag(note = "...")]`       | `#[diag(note("..."))]`     |
| `#[diag(help = "...")]`       | `#[diag(help("..."))]`     |
| field `#[diag(label = "...")]` | `#[diag(label("..."))]`   |

Primary labels on variants still use `#[diag(label = "...")]`.
