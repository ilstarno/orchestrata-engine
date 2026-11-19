use crate::capacity_matching::CapacityMatchError;

use super::{
    ControlledMatchError, WeightedMatchAssignment, WeightedMatchProblem, network::ResidualNetwork,
};

/// Find the maximum number of assignments, then minimize their total stated cost.
///
/// Cost does not make an ineligible assignment possible or leave an additional job
/// unassigned. Equal-cost plans do not carry any claim about worker preferences.
/// At most 200 jobs and 200 workers are supported. Workers' capacities count jobs,
/// not hours, and no calendars, overlaps or travel are modeled.
pub fn match_capacity_min_cost(
    problem: &WeightedMatchProblem,
) -> Result<WeightedMatchAssignment, CapacityMatchError> {
    match match_capacity_min_cost_controlled(problem, || Ok::<(), std::convert::Infallible>(())) {
        Ok(result) => Ok(result),
        Err(ControlledMatchError::InvalidProblem(error)) => Err(error),
        Err(ControlledMatchError::Cancelled(never)) => match never {},
    }
}

/// Exact weighted matching with cooperative checkpoints. An error from `check`
/// discards all local work; no partial assignment or optimality claim is returned.
/// The callback must be cheap and nonblocking. This is not a hard time limit.
pub fn match_capacity_min_cost_controlled<E>(
    problem: &WeightedMatchProblem,
    mut check: impl FnMut() -> Result<(), E>,
) -> Result<WeightedMatchAssignment, ControlledMatchError<E>> {
    check().map_err(ControlledMatchError::Cancelled)?;
    problem
        .validate()
        .map_err(ControlledMatchError::InvalidProblem)?;

    let source = 0;
    let first_job = 1;
    let first_worker = first_job + problem.eligible_workers_by_job.len();
    let sink = first_worker + problem.worker_capacities.len();
    let mut network = ResidualNetwork::new(sink + 1);
    let mut assignment_edges = Vec::with_capacity(problem.eligible_workers_by_job.len());

    for (job, eligible) in problem.eligible_workers_by_job.iter().enumerate() {
        check().map_err(ControlledMatchError::Cancelled)?;
        let node = first_job + job;
        network.add(source, node, 1, 0);
        let edges = eligible
            .iter()
            .map(|choice| network.add(node, first_worker + choice.worker, 1, choice.cost))
            .collect::<Vec<_>>();
        assignment_edges.push(edges);
    }

    for (worker, &capacity) in problem.worker_capacities.iter().enumerate() {
        network.add(first_worker + worker, sink, capacity, 0);
    }

    let mut assigned = 0;
    while network
        .augment_cheapest_controlled(source, sink, &mut check)
        .map_err(ControlledMatchError::Cancelled)?
    {
        assigned += 1;
    }

    check().map_err(ControlledMatchError::Cancelled)?;
    let mut total_cost = 0;
    let workers_by_job = assignment_edges
        .iter()
        .zip(&problem.eligible_workers_by_job)
        .enumerate()
        .map(|(job, (edges, eligible))| {
            edges.iter().zip(eligible).find_map(|(&edge, choice)| {
                if network.saturated(first_job + job, edge) {
                    total_cost += u64::from(choice.cost);
                    Some(choice.worker)
                } else {
                    None
                }
            })
        })
        .collect();
    check().map_err(ControlledMatchError::Cancelled)?;
    Ok(WeightedMatchAssignment {
        workers_by_job,
        assigned,
        total_cost,
    })
}
