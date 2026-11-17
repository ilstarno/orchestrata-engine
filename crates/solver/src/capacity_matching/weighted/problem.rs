use super::WorkerCost;
use crate::capacity_matching::{CapacityMatchError, problem::validate_inputs};

/// Each job needs one eligible worker. Each worker's capacity is a job count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedMatchProblem {
    pub worker_capacities: Vec<usize>,
    pub eligible_workers_by_job: Vec<Vec<WorkerCost>>,
}

impl WeightedMatchProblem {
    pub(super) fn validate(&self) -> Result<(), CapacityMatchError> {
        validate_inputs(
            &self.worker_capacities,
            self.eligible_workers_by_job.len(),
            |job| {
                self.eligible_workers_by_job[job]
                    .iter()
                    .map(|edge| edge.worker)
            },
        )
    }
}
