use crate::{AssignmentChange, Score};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionComparison {
    /// Scores independently checked against each plan's own model.
    /// Scores are not directly comparable if objectives or weights changed.
    pub previous_score: Score,
    pub updated_score: Score,
    /// Only changed assignments, in ascending variable order.
    pub changes: Vec<AssignmentChange>,
}
