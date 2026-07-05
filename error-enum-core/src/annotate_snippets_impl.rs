use crate::{ErrorType, Kind, Span};
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
    let footer = Vec::new();
    let uri = primary_span.uri().to_string();
    let mut slices = vec![Slice {
        source: primary_span.source_text().as_ref(),
        line_start: 1,
        origin: Some(&uri),
        annotations: vec![SourceAnnotation {
            range: (primary_span.range().start, primary_span.range().end),
            label: &primary_label,
            annotation_type: kind.into(),
        }]
        .into(),
        fold: true,
    }];
    let additional = error
        .additional()
        .map(|(span, msg, label)| (label.to_string(), (msg, span)))
        .collect::<Vec<_>>();
    for (label, (_, span)) in additional.iter() {
        let mut annotations = vec![];
        if let Some(span) = span {
            annotations.push(SourceAnnotation {
                range: (span.range().start, span.range().end),
                label: label.as_ref(),
                annotation_type: kind.into(),
            });
        }
        let source = if let Some(span) = &span {
            span.source_text().as_ref()
        } else {
            primary_span.source_text().as_ref()
        };
        slices.push(Slice {
            source,
            line_start: 1,
            origin: Some(&uri),
            annotations,
            fold: true,
        });
    }
    let snippet = Snippet {
        title,
        footer,
        slices,
        opt,
    };
    let list: DisplayList = snippet.into();
    list.to_string()
}
