use crate::Span;
use alloc::vec::Vec;

/// One group of labels sharing the same source text, ordered by first appearance.
pub(crate) struct LabelSourceGroup<S, L> {
    pub first_order: usize,
    pub source: S,
    pub entries: Vec<(S, L)>,
}

/// Group `(order, span, label)` items by [`Span::share_source_text`].
///
/// Groups are ordered by the minimum `order` (first declaration) in each group.
pub(crate) fn group_labels_by_source<S: Span, L>(
    items: Vec<(usize, S, L)>,
) -> Vec<LabelSourceGroup<S, L>> {
    let mut groups: Vec<LabelSourceGroup<S, L>> = Vec::new();
    'next: for (order, span, label) in items {
        for group in &mut groups {
            if group.source.share_source_text(&span) {
                group.entries.push((span, label));
                continue 'next;
            }
        }
        groups.push(LabelSourceGroup {
            first_order: order,
            source: span.clone(),
            entries: alloc::vec![(span, label)],
        });
    }
    groups.sort_by_key(|group| group.first_order);
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleSpan;
    use alloc::vec;

    #[test]
    fn group_same_source_merges_entries() {
        let base = SimpleSpan::new("file.rs", "alpha beta", 0, 5);
        let groups = group_labels_by_source(vec![
            (0, base.clone(), "first"),
            (1, base.with_range(6, 10), "second"),
        ]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].entries.len(), 2);
        assert_eq!(groups[0].entries[0].1, "first");
        assert_eq!(groups[0].entries[1].1, "second");
    }

    #[test]
    fn group_different_sources_orders_by_first_appearance() {
        let file_b = SimpleSpan::new("b.rs", "bbb", 0, 1);
        let file_a = SimpleSpan::new("a.rs", "aaa", 0, 1);
        let groups = group_labels_by_source(vec![
            (0, file_b, "b-first"),
            (1, file_a.clone(), "a-first"),
            (2, file_a.with_range(1, 2), "a-second"),
        ]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].source.uri().as_ref(), "b.rs");
        assert_eq!(groups[1].source.uri().as_ref(), "a.rs");
        assert_eq!(groups[1].entries.len(), 2);
    }
}
