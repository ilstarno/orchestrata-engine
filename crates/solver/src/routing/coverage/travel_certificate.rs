use super::{CourierTravelCapacity, CoverageCertificate, FlowWitness};
/// Stronger coverage-only result. Existing certificate results are unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TravelCoverageCertificate {
    pub base: CoverageCertificate,
    pub travel: FlowWitness,
    pub courier_bounds: Vec<CourierTravelCapacity>,
    pub upper_bound: usize,
    pub incumbent_covered: usize,
    pub maximum_coverage: bool,
    pub work: u32,
}
