#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStop {
    /// The next candidate would exceed the evaluation budget.
    EvaluationLimit,
    /// No move in the configured neighborhood improves the current assignment.
    NeighborhoodExhausted,
}
