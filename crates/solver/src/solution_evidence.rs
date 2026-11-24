use crate::Solution;

/// What has been established for the exact problem passed to the search.
/// Optimality/infeasibility rely on exhaustive enumeration, not an independent
/// optimality certificate. Every returned incumbent has separately checked
/// feasibility and score. Unknown never means infeasible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolutionEvidence {
    Unknown,
    VerifiedFeasible(Solution),
    ProvenOptimal(Solution),
    ProvenInfeasible,
}
