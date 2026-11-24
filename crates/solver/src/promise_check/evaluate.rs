use super::{
    DecisionBundle, PromiseError as E, PromiseRequest, VerificationOutcome,
    comparison::compare,
    planning::{model, summarize},
    validation::validate,
};
use crate::capacity_matching::weighted::{
    CertificateEvaluation, WeightedMatchProblem, solve_weighted_report,
};

/// Compare all policies against one independently checked incumbent.
/// Confirmed jobs reserve capacity before residual optimization. Their constant
/// objective contribution cannot be traded away. Certificate scope is the
/// residual problem; feasibility of the assembled whole plan is checked again.
/// Bounds: 200 jobs/resources, 8 metrics/policies, 20M verification arcs/policy.
/// No I/O, routing, temporal scheduling, live freshness or observed savings.
pub fn check_promises(request: &PromiseRequest) -> Result<DecisionBundle, E> {
    validate(request)?;
    let s = &request.snapshot;
    let incumbent = summarize(s, s.incumbent.clone())?;
    let fixed: Vec<_> = s
        .jobs
        .iter()
        .map(|j| s.commitments.iter().any(|c| c.job_id == j.id))
        .collect();
    let free: Vec<_> = (0..s.jobs.len()).filter(|&j| !fixed[j]).collect();
    let mut comparisons = Vec::with_capacity(request.policies.len());
    for policy in &request.policies {
        let metric = s
            .metrics
            .iter()
            .position(|m| m.id == policy.minimize_metric)
            .expect("validated metric");
        let full = model(s, metric);
        let mut capacities = full.worker_capacities;
        for (job, &is_fixed) in fixed.iter().enumerate() {
            if is_fixed {
                let id = s.incumbent[job].as_ref().ok_or(E::CommitmentConflict)?;
                let worker = s
                    .resources
                    .iter()
                    .position(|r| r.id == *id)
                    .ok_or(E::InvalidReference)?;
                capacities[worker] = capacities[worker]
                    .checked_sub(1)
                    .ok_or(E::CommitmentConflict)?;
            }
        }
        let residual = WeightedMatchProblem {
            worker_capacities: capacities,
            eligible_workers_by_job: free
                .iter()
                .map(|&j| full.eligible_workers_by_job[j].clone())
                .collect(),
        };
        let result =
            solve_weighted_report(&residual, Some(request.verification_work)).map_err(E::Solver)?;
        let verification = match result.certificate_evaluation() {
            CertificateEvaluation::Verified(c) => VerificationOutcome::Verified { work: c.work },
            CertificateEvaluation::WorkLimit => VerificationOutcome::WorkLimit,
            CertificateEvaluation::Rejected(e) => return Err(E::CertificateRejected(e.clone())),
            CertificateEvaluation::NotRequested => unreachable!("explicit verification budget"),
        };
        let mut proposed = s.incumbent.clone();
        for (&job, worker) in free.iter().zip(&result.assignment().workers_by_job) {
            proposed[job] = worker.map(|w| s.resources[w].id.clone());
        }
        let plan = summarize(s, proposed)?;
        comparisons.push(compare(
            s,
            &incumbent,
            policy.id.clone(),
            plan,
            verification,
        ));
    }
    Ok(DecisionBundle {
        version: 1,
        scope: super::DecisionScope::SingleSnapshotCapacityAssignment,
        estimate_basis: super::EstimateBasis::SuppliedEstimatesNotObservedSavings,
        request: request.clone(),
        incumbent,
        comparisons,
    })
}

/// Recompute and compare all typed inputs, assignments, accounting and evidence.
/// This is deterministic replay, not a signature, provenance or freshness check.
pub fn replay_decision(bundle: &DecisionBundle) -> Result<(), E> {
    if bundle.version != 1 {
        return Err(E::UnsupportedVersion);
    }
    if check_promises(&bundle.request)? != *bundle {
        return Err(E::ReplayMismatch);
    }
    Ok(())
}
