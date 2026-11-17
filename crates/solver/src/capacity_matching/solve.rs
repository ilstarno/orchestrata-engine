use super::{
    CAPACITY_MATCH_MAX_RECORDS, CapacityAssignment, CapacityMatchError, CapacityMatchProblem,
};

/// Assign as many jobs as possible subject to eligibility and worker capacity.
/// Ties depend on the input order. This model does not include travel, overlapping time slots,
/// or preferences beyond eligibility. Each successful augmenting path adds one assigned job.
pub fn match_capacity(
    problem: &CapacityMatchProblem,
) -> Result<CapacityAssignment, CapacityMatchError> {
    problem.validate()?;
    let mut jobs_by_worker = vec![Vec::new(); problem.worker_capacities.len()];
    let mut assigned = 0;
    for job in 0..problem.eligible_workers_by_job.len() {
        let mut seen = [false; CAPACITY_MATCH_MAX_RECORDS];
        if augment(problem, job, &mut jobs_by_worker, &mut seen) {
            assigned += 1;
        }
    }
    let mut workers_by_job = vec![None; problem.eligible_workers_by_job.len()];
    for (worker, jobs) in jobs_by_worker.into_iter().enumerate() {
        for job in jobs {
            workers_by_job[job] = Some(worker);
        }
    }
    Ok(CapacityAssignment {
        workers_by_job,
        assigned,
    })
}

fn augment(
    problem: &CapacityMatchProblem,
    job: usize,
    jobs_by_worker: &mut [Vec<usize>],
    seen: &mut [bool; CAPACITY_MATCH_MAX_RECORDS],
) -> bool {
    for &worker in &problem.eligible_workers_by_job[job] {
        if std::mem::replace(&mut seen[worker], true) {
            continue;
        }
        if jobs_by_worker[worker].len() < problem.worker_capacities[worker] {
            jobs_by_worker[worker].push(job);
            return true;
        }
        for position in 0..jobs_by_worker[worker].len() {
            let incumbent = jobs_by_worker[worker][position];
            if augment(problem, incumbent, jobs_by_worker, seen) {
                jobs_by_worker[worker][position] = job;
                return true;
            }
        }
    }
    false
}
