use super::{CourierClique, FlowWitness};
/// Complete coverage bound, not a travel-time or other objective certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageCertificate {
    pub upper_bound: usize,
    pub incumbent_covered: usize,
    pub maximum_coverage: bool,
    pub work: u32,
    /// Slots are couriers in input order.
    pub service: FlowWitness,
    /// Slots correspond to `cliques` in order.
    pub conflicts: FlowWitness,
    pub cliques: Vec<CourierClique>,
}
