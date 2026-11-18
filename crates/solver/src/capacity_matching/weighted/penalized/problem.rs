use super::PenaltyJob;
use crate::capacity_matching::{CapacityMatchError, problem::validate_inputs};

/// Independent unit-demand jobs with explicit eligibility, capacity and omission costs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PenaltyMatchProblem {
    pub worker_capacities: Vec<usize>,
    pub jobs: Vec<PenaltyJob>,
}

impl PenaltyMatchProblem {
    pub(super) fn validate(&self) -> Result<(), CapacityMatchError> {
        validate_inputs(&self.worker_capacities, self.jobs.len(), |job| {
            self.jobs[job]
                .eligible_workers
                .iter()
                .map(|edge| edge.worker)
        })
    }
}
