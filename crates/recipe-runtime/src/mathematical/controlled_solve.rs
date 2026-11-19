use super::{
    CompiledModel, ControlledModelError, ModelError, ModelSolution, coverage_goal::CoverageGoal,
    finish_assignment::finish_assignment,
};
use orchestrata_solver::capacity_matching::weighted::{
    ControlledMatchError, WeightedMatchProblem, match_capacity_min_cost_controlled,
};

/// Cooperatively solve a compiled coverage-then-edge-cost model. Parsing and
/// compilation are outside this boundary. Other goals are explicitly rejected.
/// Checkpoints do not guarantee a wall-clock deadline or interrupt the callback.
pub fn solve_model_controlled<E>(
    model: CompiledModel,
    mut check: impl FnMut() -> Result<(), E>,
) -> Result<ModelSolution, ControlledModelError<E>> {
    check().map_err(ControlledModelError::Cancelled)?;
    if !matches!(model.goal, CoverageGoal::CoverageThenEdgeCost) {
        return Err(ControlledModelError::UnsupportedGoal);
    }
    let mut model = model.assignment;
    let result = match_capacity_min_cost_controlled(
        &WeightedMatchProblem {
            worker_capacities: std::mem::take(&mut model.capacities),
            eligible_workers_by_job: std::mem::take(&mut model.domains),
        },
        &mut check,
    )
    .map_err(|error| match error {
        ControlledMatchError::Cancelled(reason) => ControlledModelError::Cancelled(reason),
        ControlledMatchError::InvalidProblem(_) => {
            ControlledModelError::Model(ModelError::SolverFailure)
        }
    })?;
    let solution = finish_assignment(
        model,
        result.workers_by_job,
        result.assigned,
        Some(result.total_cost),
    )
    .map_err(ControlledModelError::Model)?;
    check().map_err(ControlledModelError::Cancelled)?;
    Ok(solution)
}
