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
        if self.worker_capacities.len() > CAPACITY_MATCH_MAX_RECORDS
            || self.eligible_workers_by_job.len() > CAPACITY_MATCH_MAX_RECORDS
        {
            return Err(CapacityMatchError::TooManyRecords);
        }
        for (worker, &cap) in self.worker_capacities.iter().enumerate() {
            if cap > CAPACITY_MATCH_MAX_RECORDS {
                return Err(CapacityMatchError::InvalidCapacity { worker });
            }
        }
        for (job, eligible) in self.eligible_workers_by_job.iter().enumerate() {
            let mut seen = [false; CAPACITY_MATCH_MAX_RECORDS];
            for &worker in eligible {
                if worker >= self.worker_capacities.len() {
                    return Err(CapacityMatchError::UnknownWorker { job, worker });
                }
                if std::mem::replace(&mut seen[worker], true) {
                    return Err(CapacityMatchError::RepeatedWorker { job, worker });
                }
            }
        }
        Ok(())
    }
}
