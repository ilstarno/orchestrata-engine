/// Validation failures within a `no_overlap` rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoOverlapError {
    TooFewIntervals,
    UnknownStart(usize),
    DuplicateStart(usize),
}
