use super::{
    AssignmentError, assignment_mode::AssignmentMode, compile_capacity_json,
    compile_skills::read_records, load_assignment_recipe, skill_job::SkillJob,
    skill_match::SkillMatch, skill_named_inputs::SkillNamedInputs, skill_worker::SkillWorker,
};
use crate::{Allocation, AllocationResult, RuntimeError, UnassignedReason};
use orchestrata_solver::capacity_matching::{CapacityMatchProblem, match_capacity};

/// Allocate a batch by the XML's skill policy and maximum-coverage objective.
/// Capacities count jobs in the batch; this does not create a time schedule.
pub fn allocate_named(xml: &str, inputs: &str) -> Result<AllocationResult, RuntimeError> {
    let spec = load_assignment_recipe(xml)?;
    let AssignmentMode::BySkill(mode) = spec.mode else {
        return Err(RuntimeError::NamedSkillRecipeRequired);
    };
    let compiled = compile_capacity_json(&spec, inputs)?;
    let SkillNamedInputs(raw) =
        serde_json::from_str(inputs).map_err(|_| AssignmentError::InvalidInputs)?;
    let jobs: Vec<SkillJob> = read_records(&raw, &spec.jobs)?;
    let workers: Vec<SkillWorker> = read_records(&raw, &spec.workers)?;
    let result = match_capacity(&CapacityMatchProblem {
        worker_capacities: compiled.worker_capacities,
        eligible_workers_by_job: compiled.eligible_workers_by_job,
    })
    .map_err(RuntimeError::Solver)?;
    let allocations = jobs
        .iter()
        .zip(result.workers_by_job)
        .map(|(job, worker)| Allocation {
            job_id: job.id.clone(),
            worker_id: worker.map(|index| workers[index].id.clone()),
            unassigned_reason: worker.is_none().then(|| reason(job, &workers, mode)),
        })
        .collect();
    Ok(AllocationResult {
        assigned: result.assigned,
        output_field: spec.output,
        allocations,
    })
}

fn reason(job: &SkillJob, workers: &[SkillWorker], mode: SkillMatch) -> UnassignedReason {
    let matching = workers
        .iter()
        .filter(|worker| {
            worker
                .skills
                .iter()
                .any(|skill| mode.key(skill) == mode.key(&job.required_skill))
        })
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return UnassignedReason::NoMatchingSkill;
    }
    if matching.iter().all(|worker| !worker.available) {
        return UnassignedReason::MatchingWorkersUnavailable;
    }
    if matching
        .iter()
        .filter(|worker| worker.available)
        .all(|worker| worker.capacity == 0)
    {
        return UnassignedReason::NoAvailableCapacity;
    }
    UnassignedReason::CapacityExhausted
}
