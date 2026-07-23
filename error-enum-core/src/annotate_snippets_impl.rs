use crate::{
    label_groups::group_labels_by_source, AdditionalKind, DiagnosticKind, ErrorType, Span,
};
use alloc::{
    string::{String, ToString as _},
    vec::Vec,
};
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

impl From<AdditionalKind> for AnnotationType {
    fn from(value: AdditionalKind) -> Self {
        match value {
            AdditionalKind::Note => AnnotationType::Note,
            AdditionalKind::Help => AnnotationType::Help,
        }
    }
}
impl From<&AdditionalKind> for AnnotationType {
    fn from(value: &AdditionalKind) -> Self {
        (*value).into()
    }
}

fn is_placeholder_span<S: Span>(span: &S) -> bool {
    span.start() == span.end() && span.start() == 0 && span.uri().to_string().is_empty()
}

pub(crate) fn fmt_as_annotate_snippets<T: ErrorType + ?Sized>(
    error: &T,
    opt: FormatOptions,
) -> String {
    let primary_message = error.primary_message().to_string();
    let primary_labels = error.primary_labels();
    let code = error.code();
    let annotation_type = error.kind().as_annotate_snippets();
    let title = Annotation {
        id: Some(code.as_ref()),
        label: Some(&primary_message),
        annotation_type,
    };
    let title = Some(title);
    let mut footer_entries: Vec<(String, AdditionalKind)> = Vec::new();
    let mut ordered_labels: Vec<(usize, T::Span, (String, AnnotationType))> = Vec::new();
    let mut order = 0usize;
    for (span, label) in primary_labels.iter().cloned() {
        ordered_labels.push((order, span, (label.to_string(), annotation_type)));
        order += 1;
    }
    for (message, labels, additional_kind) in error.additional() {
        let message = message.to_string();
        let additional_annotation_type = additional_kind.into();
        let mut has_real_span = false;
        for (span, label) in labels.iter().cloned() {
            if is_placeholder_span(&span) {
                continue;
            }
            has_real_span = true;
            ordered_labels.push((order, span, (label.to_string(), additional_annotation_type)));
            order += 1;
        }
        if has_real_span {
            if !message.is_empty() && message != labels.first().1.to_string() {
                footer_entries.push((message, additional_kind));
            }
        } else if !message.is_empty() {
            footer_entries.push((message, additional_kind));
        }
    }
    let groups = group_labels_by_source(ordered_labels);
    let mut slices: Vec<Slice> = Vec::new();
    let mut stored_origins: Vec<String> = Vec::new();
    let mut pending_slices: Vec<(&str, Vec<SourceAnnotation>)> = Vec::new();
    for group in &groups {
        let annotations: Vec<SourceAnnotation> = group
            .entries
            .iter()
            .map(|(span, (label, annotation_type))| SourceAnnotation {
                range: (span.range().start, span.range().end),
                label: label.as_str(),
                annotation_type: *annotation_type,
            })
            .collect();
        stored_origins.push(group.source.uri().to_string());
        pending_slices.push((group.source.source_text().as_ref(), annotations));
    }
    for (index, (source, annotations)) in pending_slices.into_iter().enumerate() {
        slices.push(Slice {
            source,
            line_start: 1,
            origin: Some(stored_origins[index].as_str()),
            annotations,
            fold: true,
        });
    }
    let footer = footer_entries
        .iter()
        .map(|(message, additional_kind)| Annotation {
            id: None,
            label: Some(message.as_str()),
            annotation_type: additional_kind.into(),
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
