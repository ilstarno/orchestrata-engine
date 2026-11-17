use serde::Deserialize;

use super::CapacityMatchError;

pub const CAPACITY_MATCH_MAX_RECORDS: usize = 200;

/// Jobs are independent and each needs one eligible worker. Worker capacity is a job count.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapacityMatchProblem {
    pub worker_capacities: Vec<usize>,
    pub eligible_workers_by_job: Vec<Vec<usize>>,
}

impl CapacityMatchProblem {
    pub(super) fn validate(&self) -> Result<(), CapacityMatchError> {
        validate_inputs(
            &self.worker_capacities,
            self.eligible_workers_by_job.len(),
            |job| self.eligible_workers_by_job[job].iter().copied(),
        )
    }
}

pub(super) fn validate_inputs<I: Iterator<Item = usize>>(
    worker_capacities: &[usize],
    job_count: usize,
    eligible_workers: impl Fn(usize) -> I,
) -> Result<(), CapacityMatchError> {
    if worker_capacities.len() > CAPACITY_MATCH_MAX_RECORDS
        || job_count > CAPACITY_MATCH_MAX_RECORDS
    {
        return Err(CapacityMatchError::TooManyRecords);
    }
    for (worker, &cap) in worker_capacities.iter().enumerate() {
        if cap > CAPACITY_MATCH_MAX_RECORDS {
            return Err(CapacityMatchError::InvalidCapacity { worker });
        }
    }
    for job in 0..job_count {
        let mut seen = [false; CAPACITY_MATCH_MAX_RECORDS];
        for worker in eligible_workers(job) {
            if worker >= worker_capacities.len() {
                return Err(CapacityMatchError::UnknownWorker { job, worker });
            }
            if std::mem::replace(&mut seen[worker], true) {
                return Err(CapacityMatchError::RepeatedWorker { job, worker });
            }
        }
    }
    Ok(())
}
