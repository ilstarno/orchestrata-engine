use super::{PenaltyMatchAssignment, PenaltyMatchProblem};
use crate::capacity_matching::{CapacityMatchError, weighted::network::ResidualNetwork};

/// Minimize service costs plus the explicitly supplied costs of unassigned jobs.
///
/// Unlike maximum-cardinality matching, this may leave a feasible job unassigned.
/// No coverage objective or hidden tie preference is applied. Replaying the same
/// ordered input is deterministic; equal-cost optima need not maximize coverage.
/// Eligibility and count capacities are hard constraints. Limits are 200 jobs,
/// 200 workers and capacity 200 per worker. No schedules or variable demands.
pub fn match_capacity_with_penalties(
    problem: &PenaltyMatchProblem,
) -> Result<PenaltyMatchAssignment, CapacityMatchError> {
    problem.validate()?;
    let first_worker = 1 + problem.jobs.len();
    let sink = first_worker + problem.worker_capacities.len();
    let mut network = ResidualNetwork::new(sink + 1);
    let mut assignment_edges = Vec::with_capacity(problem.jobs.len());
    for (job, input) in problem.jobs.iter().enumerate() {
        let node = job + 1;
        network.add(0, node, 1, 0);
        // The direct edge represents paying this job's omission cost.
        network.add(node, sink, 1, input.unassigned_cost);
        assignment_edges.push(
            input
                .eligible_workers
                .iter()
                .map(|choice| network.add(node, first_worker + choice.worker, 1, choice.cost))
                .collect::<Vec<_>>(),
        );
    }
    for (worker, &capacity) in problem.worker_capacities.iter().enumerate() {
        network.add(first_worker + worker, sink, capacity, 0);
    }
    // Every job has a direct sink edge, so exactly one unit per job can flow.
    while network.augment_cheapest(0, sink) {}
    let mut service_cost = 0;
    let mut unassigned_cost = 0;
    let mut assigned = 0;
    let workers_by_job = problem
        .jobs
        .iter()
        .zip(assignment_edges)
        .enumerate()
        .map(|(job, (input, edges))| {
            let worker = input
                .eligible_workers
                .iter()
                .zip(edges)
                .find_map(|(choice, edge)| {
                    if network.saturated(job + 1, edge) {
                        service_cost += u64::from(choice.cost);
                        assigned += 1;
                        Some(choice.worker)
                    } else {
                        None
                    }
                });
            if worker.is_none() {
                unassigned_cost += u64::from(input.unassigned_cost);
            }
            worker
        })
        .collect();
    Ok(PenaltyMatchAssignment {
        workers_by_job,
        assigned,
        service_cost,
        unassigned_cost,
        total_cost: service_cost + unassigned_cost,
    })
}
