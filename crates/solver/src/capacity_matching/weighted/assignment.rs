/// One eligible worker or no assignment per job in the supplied order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedMatchAssignment {
    pub workers_by_job: Vec<Option<usize>>,
    pub assigned: usize,
    pub total_cost: u64,
}
