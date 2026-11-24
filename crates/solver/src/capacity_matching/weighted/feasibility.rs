use super::{WeightedMatchAssignment, WeightedMatchProblem, WeightedVerificationError};

/// Independently checks eligibility, capacities, and exact assignment accounting.
/// Does not check maximum coverage or minimum cost.
pub fn verify_weighted_feasibility(
    problem: &WeightedMatchProblem,
    assignment: &WeightedMatchAssignment,
) -> Result<(), WeightedVerificationError> {
    use WeightedVerificationError as E;
    problem.validate().map_err(E::InvalidProblem)?;
    let jobs = problem.eligible_workers_by_job.len();
    let workers = problem.worker_capacities.len();
    if assignment.workers_by_job.len() != jobs {
        return Err(E::InvalidAssignment);
    }
    let mut loads = vec![0usize; workers];
    let mut count = 0;
    let mut cost = 0u64;
    for (job, selected) in assignment.workers_by_job.iter().enumerate() {
        if let Some(worker) = selected {
            let choice = problem.eligible_workers_by_job[job]
                .iter()
                .find(|e| e.worker == *worker)
                .ok_or(E::InvalidAssignment)?;
            loads[*worker] += 1;
            if loads[*worker] > problem.worker_capacities[*worker] {
                return Err(E::InvalidAssignment);
            }
            count += 1;
            cost += u64::from(choice.cost);
        }
    }
    if count != assignment.assigned {
        return Err(E::CountMismatch);
    }
    if cost != assignment.total_cost {
        return Err(E::CostMismatch);
    }
    Ok(())
}
