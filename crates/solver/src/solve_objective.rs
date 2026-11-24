/// Objective of the finite-domain reference solver only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveObjective {
    /// Satisfy every hard constraint, then maximize Score::soft (negative penalty).
    MaximizeSoftScoreSubjectToHardConstraints,
}
