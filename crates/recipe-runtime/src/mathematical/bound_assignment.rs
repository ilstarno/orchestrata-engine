use super::record_id::RecordId;
use orchestrata_solver::capacity_matching::weighted::WorkerCost;
#[derive(Debug)]
pub(super) struct BoundAssignment {
    pub items: Vec<RecordId>,
    pub resources: Vec<RecordId>,
    pub capacities: Vec<usize>,
    pub domains: Vec<Vec<WorkerCost>>,
    pub nullable: bool,
}
