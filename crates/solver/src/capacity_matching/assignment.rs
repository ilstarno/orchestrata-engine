use serde::Serialize;

/// One worker index or no assignment for each job, in the original job order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapacityAssignment {
    pub workers_by_job: Vec<Option<usize>>,
    pub assigned: usize,
}
