use crate::{ErrorEnum, Kind};
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Snippet},
};

impl From<Kind> for AnnotationType {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Error => AnnotationType::Error,
            Kind::Warn => AnnotationType::Warning,
        }
    }
}

pub(crate) fn fmt_as_annotate_snippets<T: ErrorEnum + ?Sized>(
    error: &T,
    opt: FormatOptions,
) -> String {
    let primary_message = error.primary_message().to_string();
    let kind = error.kind();
    let title = Annotation {
        id: Some(error.code()),
        label: Some(&primary_message),
        annotation_type: kind.into(),
    };
    let title = Some(title);
    let footer = Vec::new();
    let slices = Vec::new();
    let snippet = Snippet {
        title,
        footer,
        slices,
        opt,
    };
    let list: DisplayList = snippet.into();
    list.to_string()
}
