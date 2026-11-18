use super::super::WorkerCost;

/// Costs must share one caller-defined nonnegative unit, such as cents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PenaltyJob {
    pub eligible_workers: Vec<WorkerCost>,
    /// Cost incurred exactly once if this job is left unassigned.
    pub unassigned_cost: u32,
}
