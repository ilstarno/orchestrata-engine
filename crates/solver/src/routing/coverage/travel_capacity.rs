use super::super::LocationId;
/// Inspectable optimistic lower bounds indexed by admitted shipment count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourierTravelCapacity {
    pub courier: usize,
    pub capacity: usize,
    pub depot_only: usize,
    pub representatives: Vec<(LocationId, usize)>,
    pub minimum_outbound: Option<u128>,
    pub minimum_return: Option<u128>,
    pub minimum_between: Option<u128>,
    pub service_prefixes: Vec<u128>,
    pub distinct_nodes: Vec<usize>,
    pub travel_lower_bounds: Vec<u128>,
}
