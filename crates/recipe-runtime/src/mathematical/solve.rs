use super::{CompiledModel, ModelAssignment, ModelError, ModelSolution};
use orchestrata_solver::capacity_matching::{
    CapacityMatchProblem, match_capacity,
    weighted::{WeightedMatchProblem, match_capacity_min_cost},
};

pub fn solve_model(model: CompiledModel) -> Result<ModelSolution, ModelError> {
    let (workers, assigned, cost) = if model.minimize_cost {
        let result = match_capacity_min_cost(&WeightedMatchProblem {
            worker_capacities: model.capacities,
            eligible_workers_by_job: model.domains,
        })
        .map_err(|_| ModelError::SolverFailure)?;
        (
            result.workers_by_job,
            result.assigned,
            Some(result.total_cost),
        )
    } else {
        let result = match_capacity(&CapacityMatchProblem {
            worker_capacities: model.capacities,
            eligible_workers_by_job: model
                .domains
                .iter()
                .map(|edges| edges.iter().map(|e| e.worker).collect())
                .collect(),
        })
        .map_err(|_| ModelError::SolverFailure)?;
        (result.workers_by_job, result.assigned, None)
    };
    if !model.nullable && assigned != model.items.len() {
        return Err(ModelError::Infeasible);
    }
    Ok(ModelSolution {
        assigned,
        minimized_edge_sum: cost,
        assignments: model
            .items
            .into_iter()
            .zip(workers)
            .map(|(item_id, r)| ModelAssignment {
                item_id,
                resource_id: r.map(|index| model.resources[index].clone()),
            })
            .collect(),
    })
}
