/// Independently checked maximum coverage and minimum cost at that coverage.
/// Valid only for the exact problem and assignment supplied to the verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedMatchCertificate {
    pub maximum_assigned: usize,
    pub minimum_cost: u64,
    /// Residual arcs examined during reachability and negative-cycle checking.
    pub work: u64,
}
