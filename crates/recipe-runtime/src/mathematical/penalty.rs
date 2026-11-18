use super::{
    CompiledPenaltyModel, ModelAssignment, ModelError, PenaltyModelSolution, PenaltyModelSpec,
    objective::Objective,
};
use orchestrata_solver::capacity_matching::weighted::penalized::{
    PenaltyJob, PenaltyMatchProblem, match_capacity_with_penalties,
};

pub fn parse_penalty_model(xml: &str) -> Result<PenaltyModelSpec, ModelError> {
    Ok(PenaltyModelSpec(super::parse::parse_version(xml, "2")?))
}
pub fn compile_penalty_model(
    spec: &PenaltyModelSpec,
    records: &str,
) -> Result<CompiledPenaltyModel, ModelError> {
    let [Objective::MinTotalCost { unit, .. }] = spec.0.objectives.as_slice() else {
        return Err(ModelError::UnsupportedSemantics);
    };
    let (model, penalties) = super::compile::compile_bound(&spec.0, records)?;
    Ok(CompiledPenaltyModel {
        model,
        penalties,
        unit: unit.clone(),
    })
}
pub fn solve_penalty_model(
    compiled: CompiledPenaltyModel,
) -> Result<PenaltyModelSolution, ModelError> {
    let model = compiled.model;
    let result = match_capacity_with_penalties(&PenaltyMatchProblem {
        worker_capacities: model.capacities,
        jobs: model
            .domains
            .into_iter()
            .zip(compiled.penalties)
            .map(|(eligible_workers, unassigned_cost)| PenaltyJob {
                eligible_workers,
                unassigned_cost,
            })
            .collect(),
    })
    .map_err(|_| ModelError::SolverFailure)?;
    Ok(PenaltyModelSolution {
        schema_version: 2,
        objective: "service-plus-unassigned-cost",
        unit: compiled.unit,
        assigned: result.assigned,
        service_cost: result.service_cost,
        unassigned_cost: result.unassigned_cost,
        total_cost: result.total_cost,
        assignments: model
            .items
            .into_iter()
            .zip(result.workers_by_job)
            .map(|(item_id, worker)| ModelAssignment {
                item_id: item_id.into_string(),
                resource_id: worker.map(|w| model.resources[w].clone().into_string()),
            })
            .collect(),
    })
}
pub fn run_penalty_model(xml: &str, records: &str) -> Result<PenaltyModelSolution, ModelError> {
    solve_penalty_model(compile_penalty_model(&parse_penalty_model(xml)?, records)?)
}
