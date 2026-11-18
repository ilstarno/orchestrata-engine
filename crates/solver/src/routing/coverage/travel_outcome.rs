use super::TravelCoverageCertificate;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TravelCoverageOutcome {
    Complete(Box<TravelCoverageCertificate>),
    /// A partial stronger check never reports an optimality claim.
    Incomplete {
        work: u32,
    },
}
