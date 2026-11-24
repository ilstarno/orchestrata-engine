use crate::capacity_matching::weighted::WeightedVerificationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromiseError {
    UnsupportedVersion,
    LimitExceeded,
    InvalidIdentity,
    InvalidReference,
    WrongEstimateCount,
    InvalidIncumbent(WeightedVerificationError),
    CommitmentConflict,
    Solver(WeightedVerificationError),
    CertificateRejected(WeightedVerificationError),
    ReplayMismatch,
}
