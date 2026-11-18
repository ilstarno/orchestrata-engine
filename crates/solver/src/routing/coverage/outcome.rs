use super::CoverageCertificate;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageOutcome {
    Complete(Box<CoverageCertificate>),
    /// No bound or optimality claim is available when the budget expires.
    Incomplete {
        work: u32,
    },
}
