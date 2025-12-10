use crate::{ErrorType, Kind, Span};
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
    let primary_span = error.primary_span();
    let kind = error.kind();
    let title = Annotation {
        id: Some(error.code()),
        label: Some(&primary_message),
        annotation_type: kind.into(),
    };
    let title = Some(title);
    let footer = Vec::new();
    let uri = primary_span.uri().to_string();
    let slices = [Slice {
        source: primary_span.source_text().as_ref(),
        line_start: 1,
        origin: Some(&uri),
        annotations: [SourceAnnotation {
            range: (primary_span.range().start, primary_span.range().end),
            label: &primary_label,
            annotation_type: kind.into(),
        }]
        .into(),
        fold: true,
    }]
    .into();
    let snippet = Snippet {
        title,
        footer,
        slices,
        opt,
    };
    let list: DisplayList = snippet.into();
    list.to_string()
}
