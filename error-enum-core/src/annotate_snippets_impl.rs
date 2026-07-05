use crate::{AdditionalKind, ErrorType, Kind, Span};
use alloc::{
    string::{String, ToString as _},
    vec,
    vec::Vec,
};
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

impl From<Kind> for AnnotationType {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Error => AnnotationType::Error,
            Kind::Warn => AnnotationType::Warning,
        }
    }
}

struct StoredAdditional<S> {
    span: S,
    origin: String,
    annotation_label: String,
    footer_message: Option<String>,
}

pub(crate) fn fmt_as_annotate_snippets<T: ErrorType + ?Sized>(
    error: &T,
    opt: FormatOptions,
) -> String {
    let primary_message = error.primary_message().to_string();
    let primary_label = error.primary_label().to_string();
    let primary_span = error.primary_span().unwrap_or_default();
    let kind = error.kind();
    let title = Annotation {
        id: Some(error.code()),
        label: Some(&primary_message),
        annotation_type: kind.into(),
    };
    let title = Some(title);
    let mut footer_messages = Vec::new();
    let mut stored = Vec::new();
    for (span, message, label, additional_kind) in error.additional() {
        let message = message.to_string();
        let label = label.to_string();
        if span.is_none() {
            if !message.is_empty() {
                footer_messages.push(message);
            }
            continue;
        }
        let span = span.unwrap_or_default();
        let annotation_label = if !label.is_empty() {
            label
        } else if matches!(additional_kind, AdditionalKind::Label) {
            continue;
        } else {
            message.clone()
        };
        let footer_message = if !message.is_empty() && message != annotation_label {
            Some(message)
        } else {
            None
        };
        let origin = span.uri().to_string();
        stored.push(StoredAdditional {
            span,
            origin,
            annotation_label,
            footer_message,
        });
    }
    let uri = primary_span.uri().to_string();
    let mut slices = vec![Slice {
        source: primary_span.source_text().as_ref(),
        line_start: 1,
        origin: Some(&uri),
        annotations: vec![SourceAnnotation {
            range: (primary_span.range().start, primary_span.range().end),
            label: &primary_label,
            annotation_type: kind.into(),
        }],
        fold: true,
    }];
    for item in &stored {
        if let Some(message) = &item.footer_message {
            footer_messages.push(message.clone());
        }
        slices.push(Slice {
            source: item.span.source_text().as_ref(),
            line_start: 1,
            origin: Some(&item.origin),
            annotations: vec![SourceAnnotation {
                range: (item.span.range().start, item.span.range().end),
                label: &item.annotation_label,
                annotation_type: kind.into(),
            }],
            fold: true,
        });
    }
    let footer = footer_messages
        .iter()
        .map(|message| Annotation {
            id: None,
            label: Some(message.as_str()),
            annotation_type: AnnotationType::Note,
        })
        .collect();
    let snippet = Snippet {
        title,
        footer,
        slices,
        opt,
    };
    let list: DisplayList = snippet.into();
    list.to_string()
}
