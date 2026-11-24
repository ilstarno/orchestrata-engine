use super::{
    AssignmentPlan, ChangeKind, PlanChange, PlanEvidence, PlanningSnapshot, PolicyComparison,
    VerificationOutcome,
};

pub(super) fn compare(
    s: &PlanningSnapshot,
    incumbent: &AssignmentPlan,
    policy_id: String,
    plan: AssignmentPlan,
    verification: VerificationOutcome,
) -> PolicyComparison {
    let changes = s
        .jobs
        .iter()
        .zip(&incumbent.assignments)
        .zip(&plan.assignments)
        .filter_map(|((job, previous), proposed)| {
            if previous == proposed {
                return None;
            }
            let kind = match (previous, proposed) {
                (None, Some(_)) => ChangeKind::Assigned,
                (Some(_), None) => ChangeKind::Unassigned,
                _ => ChangeKind::Moved,
            };
            Some(PlanChange {
                job_id: job.id.clone(),
                previous: previous.clone(),
                proposed: proposed.clone(),
                kind,
            })
        })
        .collect();
    let unassigned_job_ids = s
        .jobs
        .iter()
        .zip(&plan.assignments)
        .filter(|(_, a)| a.is_none())
        .map(|(j, _)| j.id.clone())
        .collect();
    let evidence = match verification {
        VerificationOutcome::Verified { .. } => PlanEvidence::ProvenOptimalUnderCommitments,
        VerificationOutcome::WorkLimit => PlanEvidence::VerifiedFeasibleOptimalityUnknown,
    };
    PolicyComparison {
        policy_id,
        assigned_delta: plan.assigned as i64 - incumbent.assigned as i64,
        estimate_deltas: plan
            .estimate_totals
            .iter()
            .zip(&incumbent.estimate_totals)
            .map(|(a, b)| *a as i64 - *b as i64)
            .collect(),
        plan,
        changes,
        unassigned_job_ids,
        evidence,
        verification,
    }
}
