use crate::capacity_matching::CapacityMatchError;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeightedVerificationError {
    InvalidProblem(CapacityMatchError),
    InvalidAssignment,
    CountMismatch,
    CostMismatch,
    NotMaximumCoverage,
    NotMinimumCost,
    WorkLimit,
}
