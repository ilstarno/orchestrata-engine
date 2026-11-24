use super::{
    WeightedMatchAssignment, WeightedMatchCertificate, WeightedMatchProblem,
    WeightedVerificationError,
};
use std::collections::VecDeque;

/// Checks feasibility, exact accounting, maximum cardinality and minimum cost
/// without calling the solver or using its network, potentials or cached state.
///
/// Uses the input's existing 200-job/200-worker limits. `max_work` bounds residual
/// arc examinations, not validation/allocation time or a wall-clock deadline.
/// Exhausting the budget returns an error, never an optimality certificate.
pub fn verify_weighted_assignment(
    problem: &WeightedMatchProblem,
    assignment: &WeightedMatchAssignment,
    max_work: u64,
) -> Result<WeightedMatchCertificate, WeightedVerificationError> {
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
    let source = jobs + workers;
    let sink = source + 1;
    let nodes = sink + 1;
    let mut residual = vec![Vec::<(usize, i64)>::new(); nodes];
    for (job, edges) in problem.eligible_workers_by_job.iter().enumerate() {
        let selected = assignment.workers_by_job[job];
        if selected.is_some() {
            residual[job].push((source, 0));
        } else {
            residual[source].push((job, 0));
        }
        for edge in edges {
            let worker = jobs + edge.worker;
            if selected == Some(edge.worker) {
                residual[worker].push((job, -i64::from(edge.cost)));
            } else {
                residual[job].push((worker, i64::from(edge.cost)));
            }
        }
    }
    for (worker, &load) in loads.iter().enumerate() {
        if load < problem.worker_capacities[worker] {
            residual[jobs + worker].push((sink, 0));
        }
        if load > 0 {
            residual[sink].push((jobs + worker, 0));
        }
    }
    let mut work = 0;
    let mut seen = vec![false; nodes];
    seen[source] = true;
    let mut queue = VecDeque::from([source]);
    while let Some(node) = queue.pop_front() {
        for &(to, _) in &residual[node] {
            charge(&mut work, max_work)?;
            if to == sink {
                return Err(E::NotMaximumCoverage);
            }
            if !seen[to] {
                seen[to] = true;
                queue.push_back(to);
            }
        }
    }
    // A virtual zero-cost source reaches every vertex. Bellman-Ford therefore
    // detects cost-improving cycles even outside the source's reachable region.
    // Distances fit i128: at most 402 passes over a bounded graph with u32 costs.
    let mut distance = vec![0i128; nodes];
    for pass in 0..nodes {
        let mut changed = false;
        for (from, edges) in residual.iter().enumerate() {
            for &(to, weight) in edges {
                charge(&mut work, max_work)?;
                let next = distance[from] + i128::from(weight);
                if next < distance[to] {
                    distance[to] = next;
                    changed = true;
                    if pass + 1 == nodes {
                        return Err(E::NotMinimumCost);
                    }
                }
            }
        }
        if !changed {
            return Ok(WeightedMatchCertificate {
                maximum_assigned: count,
                minimum_cost: cost,
                work,
            });
        }
    }
    unreachable!("a final-pass relaxation returns above")
}
fn charge(work: &mut u64, limit: u64) -> Result<(), WeightedVerificationError> {
    if *work == limit {
        return Err(WeightedVerificationError::WorkLimit);
    }
    *work += 1;
    Ok(())
}
