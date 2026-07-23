/// A label attached to a span.
pub type SpannedLabel<S, L> = (S, L);

/// Non-empty labels for a diagnostic unit; index `0` is the primary label.
pub type LabelVec1<S, L> = mitsein::vec1::Vec1<SpannedLabel<S, L>>;
