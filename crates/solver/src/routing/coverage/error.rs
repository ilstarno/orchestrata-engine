use super::super::RouteVerificationError;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageError {
    LimitExceeded,
    InvalidProblem(RouteVerificationError),
    InvalidIncumbent(RouteVerificationError),
    InconsistentBound,
}
