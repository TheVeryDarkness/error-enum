# Node Attributes

| Attribute                                 | Description                                                                |
| ----------------------------------------- | -------------------------------------------------------------------------- |
| `#[diag(kind   = $kind:lit_str)]`         | `$kind` is either `"error"` or `"warn"`. Default is `"error"`.             |
| `#[diag(number = $number:lit_int)]`       | `$number` is the error number suffix.                                      |
| `#[diag(msg    = $msg:lit_str)]`          | `$msg` is the error message.                                               |
| `#[diag(note   = $note:lit_str)]`         | `$note` is an additional note message.                                     |
| `#[diag(help   = $help:lit_str)]`         | `$help` is an additional help message.                                     |
| `#[diag(span_type = $span_type:lit_str)]` | `$span_type` is the type of the span. Default is `error_enum::SimpleSpan`. |

# Field Attributes

| Attribute                         | Description                                                |
| --------------------------------- | ---------------------------------------------------------- |
| `#[diag(span)]`                   | Mark this field as the primary span of this error variant. |
| `#[diag(note = $note:lit_str)]`   | `$note` is an additional note message on the span.         |
| `#[diag(help = $help:lit_str)]`   | `$help` is an additional help message on the span.         |
| `#[diag(label = $label:lit_str)]` | `$label` is the label message on the span.                 |
