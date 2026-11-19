use super::{CompiledModel, ModelError, ModelSolution, coverage_goal::CoverageGoal};
use orchestrata_solver::capacity_matching::{
    CapacityMatchProblem, match_capacity,
    weighted::{WeightedMatchProblem, match_capacity_min_cost},
};

pub fn solve_model(model: CompiledModel) -> Result<ModelSolution, ModelError> {
    let goal = model.goal;
    let mut model = model.assignment;
    let (workers, assigned, cost) = match goal {
        CoverageGoal::CoverageThenEdgeCost => {
            let result = match_capacity_min_cost(&WeightedMatchProblem {
                worker_capacities: std::mem::take(&mut model.capacities),
                eligible_workers_by_job: std::mem::take(&mut model.domains),
            })
            .map_err(|_| ModelError::SolverFailure)?;
            (
                result.workers_by_job,
                result.assigned,
                Some(result.total_cost),
            )
        }
        CoverageGoal::MaximumCoverage => {
            let result = match_capacity(&CapacityMatchProblem {
                worker_capacities: std::mem::take(&mut model.capacities),
                eligible_workers_by_job: model
                    .domains
                    .iter()
                    .map(|edges| edges.iter().map(|e| e.worker).collect())
                    .collect(),
            })
            .map_err(|_| ModelError::SolverFailure)?;
            (result.workers_by_job, result.assigned, None)
        }
    };
    super::finish_assignment::finish_assignment(model, workers, assigned, cost)
}
