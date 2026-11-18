#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Objective {
    MaxAssigned,
    MinEdgeSum(String),
}
