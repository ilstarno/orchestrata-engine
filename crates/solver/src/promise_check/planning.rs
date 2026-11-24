use super::{AssignmentPlan, PlanningSnapshot, PromiseError};
use crate::capacity_matching::weighted::{
    WeightedMatchAssignment, WeightedMatchProblem, WorkerCost, verify_weighted_feasibility,
};

/// Called only after structural validation. All indices refer to declared order.
pub(super) fn model(s: &PlanningSnapshot, metric: usize) -> WeightedMatchProblem {
    WeightedMatchProblem {
        worker_capacities: s.resources.iter().map(|r| r.capacity).collect(),
        eligible_workers_by_job: s
            .jobs
            .iter()
            .map(|job| {
                job.options
                    .iter()
                    .map(|option| WorkerCost {
                        worker: s
                            .resources
                            .iter()
                            .position(|r| r.id == option.resource_id)
                            .expect("validated resource"),
                        cost: option.estimates[metric],
                    })
                    .collect()
            })
            .collect(),
    }
}

pub(super) fn assignment(
    s: &PlanningSnapshot,
    values: &[Option<String>],
    metric: usize,
) -> Result<WeightedMatchAssignment, PromiseError> {
    let mut total_cost = 0;
    let mut workers_by_job = Vec::with_capacity(values.len());
    for (job, value) in s.jobs.iter().zip(values) {
        if let Some(id) = value {
            let option = job
                .options
                .iter()
                .find(|o| o.resource_id == *id)
                .ok_or(PromiseError::InvalidReference)?;
            total_cost += u64::from(option.estimates[metric]);
            workers_by_job.push(Some(
                s.resources
                    .iter()
                    .position(|r| r.id == *id)
                    .ok_or(PromiseError::InvalidReference)?,
            ));
        } else {
            workers_by_job.push(None);
        }
    }
    Ok(WeightedMatchAssignment {
        assigned: workers_by_job.iter().flatten().count(),
        workers_by_job,
        total_cost,
    })
}

pub(super) fn summarize(
    s: &PlanningSnapshot,
    values: Vec<Option<String>>,
) -> Result<AssignmentPlan, PromiseError> {
    let mut totals = Vec::with_capacity(s.metrics.len());
    for metric in 0..s.metrics.len() {
        let a = assignment(s, &values, metric)?;
        verify_weighted_feasibility(&model(s, metric), &a)
            .map_err(PromiseError::InvalidIncumbent)?;
        totals.push(a.total_cost);
    }
    Ok(AssignmentPlan {
        assigned: values.iter().flatten().count(),
        assignments: values,
        estimate_totals: totals,
    })
}
