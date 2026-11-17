use crate::capacity_matching::CapacityMatchError;

use super::{WeightedMatchAssignment, WeightedMatchProblem, network::ResidualNetwork};

/// Find the maximum number of assignments, then minimize their total stated cost.
///
/// Cost does not make an ineligible assignment possible or leave an additional job
/// unassigned. Equal-cost plans do not carry any claim about worker preferences.
/// At most 200 jobs and 200 workers are supported. Workers' capacities count jobs,
/// not hours, and no calendars, overlaps or travel are modeled.
pub fn match_capacity_min_cost(
    problem: &WeightedMatchProblem,
) -> Result<WeightedMatchAssignment, CapacityMatchError> {
    problem.validate()?;

    let source = 0;
    let first_job = 1;
    let first_worker = first_job + problem.eligible_workers_by_job.len();
    let sink = first_worker + problem.worker_capacities.len();
    let mut network = ResidualNetwork::new(sink + 1);
    let mut assignment_edges = Vec::with_capacity(problem.eligible_workers_by_job.len());

    for (job, eligible) in problem.eligible_workers_by_job.iter().enumerate() {
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
    while network.augment_cheapest(source, sink) {
        assigned += 1;
    }

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
    Ok(WeightedMatchAssignment {
        workers_by_job,
        assigned,
        total_cost,
    })
}
