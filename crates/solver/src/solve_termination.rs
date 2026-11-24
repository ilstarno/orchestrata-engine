/// Why the finite-domain reference search stopped, separate from its evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveTermination {
    SearchExhausted,
    EvaluationLimit,
}
