#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStop {
    /// The next candidate would exceed the evaluation budget.
    EvaluationLimit,
    /// No single-variable replacement improves the current assignment.
    NeighborhoodExhausted,
}
