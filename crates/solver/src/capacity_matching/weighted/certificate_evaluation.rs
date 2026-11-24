use super::{WeightedMatchCertificate, WeightedVerificationError};

/// Certificate evaluation, separate from solver execution.
/// Only Verified establishes the declared weighted objective.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateEvaluation {
    NotRequested,
    WorkLimit,
    Verified(WeightedMatchCertificate),
    Rejected(WeightedVerificationError),
}
