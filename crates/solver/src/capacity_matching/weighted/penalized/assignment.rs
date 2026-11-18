#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PenaltyMatchAssignment {
    pub workers_by_job: Vec<Option<usize>>,
    pub assigned: usize,
    pub service_cost: u64,
    pub unassigned_cost: u64,
    pub total_cost: u64,
}
