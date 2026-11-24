use orchestrata_solver::promise_check::*;

fn request() -> PromiseRequest {
    serde_json::from_str(include_str!(
        "../../fixtures/promise-check/maintenance.json"
    ))
    .unwrap()
}

// Independent exhaustive oracle over original options and capacities. It does
// not use residual models, the weighted solver, or certificate implementation.
fn exact(request: &PromiseRequest, metric: usize) -> (usize, u64) {
    let s = &request.snapshot;
    let count: usize = s.jobs.iter().map(|j| j.options.len() + 1).product();
    let mut best = (0, u64::MAX);
    for code in 0..count {
        let mut code = code;
        let mut loads = vec![0; s.resources.len()];
        let mut assigned = 0;
        let mut cost = 0;
        let mut valid = true;
        for job in &s.jobs {
            let choice = code % (job.options.len() + 1);
            code /= job.options.len() + 1;
            let option = job.options.get(choice);
            if let Some(pin) = s.commitments.iter().find(|p| p.job_id == job.id)
                && option.map(|o| &o.resource_id) != Some(&pin.resource_id)
            {
                valid = false;
            }
            if let Some(option) = option {
                let worker = s
                    .resources
                    .iter()
                    .position(|r| r.id == option.resource_id)
                    .unwrap();
                loads[worker] += 1;
                if loads[worker] > s.resources[worker].capacity {
                    valid = false;
                }
                assigned += 1;
                cost += u64::from(option.estimates[metric]);
            }
        }
        if valid && (assigned > best.0 || (assigned == best.0 && cost < best.1)) {
            best = (assigned, cost);
        }
    }
    best
}

#[test]
fn actual_incumbent_tradeoffs_and_commitments_match_exact_oracle() {
    let r = request();
    let bundle = check_promises(&r).unwrap();
    assert_eq!(bundle.incumbent.assigned, 4);
    assert_eq!(bundle.incumbent.estimate_totals, [33000, 73]);
    let cost = &bundle.comparisons[0];
    let delay = &bundle.comparisons[1];
    assert_eq!(cost.plan.estimate_totals, [26000, 205]);
    assert_eq!(delay.plan.estimate_totals, [39000, 108]);
    assert_eq!(cost.estimate_deltas, [-7000, 132]);
    assert_eq!(delay.estimate_deltas, [6000, 35]);
    assert_eq!(
        cost.changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Moved)
            .count(),
        2
    );
    assert_eq!(
        cost.changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Unassigned)
            .count(),
        1
    );
    assert_eq!(cost.unassigned_job_ids, ["repair-103", "specialist-107"]);
    for (metric, c) in bundle.comparisons.iter().enumerate() {
        assert_eq!(
            (c.plan.assigned, c.plan.estimate_totals[metric]),
            exact(&r, metric)
        );
        assert_eq!(c.plan.assignments[0], Some("crew-a".into()));
        assert_eq!(c.evidence, PlanEvidence::ProvenOptimalUnderCommitments);
        assert_eq!(c.assigned_delta, 1);
    }
    replay_decision(&bundle).unwrap();
    assert_eq!(bundle, check_promises(&r).unwrap());
}

#[test]
fn varied_metrics_and_additional_pins_match_independent_enumeration() {
    for seed in 0..32u32 {
        let mut r = request();
        for (j, job) in r.snapshot.jobs.iter_mut().enumerate() {
            for (a, option) in job.options.iter_mut().enumerate() {
                option.estimates = vec![
                    (seed * 17 + j as u32 * 23 + a as u32 * 3) % 71,
                    (seed * 13 + j as u32 * 5 + a as u32 * 11) % 43,
                ];
            }
        }
        if seed % 2 == 0 {
            r.snapshot.commitments.push(Commitment {
                job_id: "repair-103".into(),
                resource_id: "crew-a".into(),
            });
        }
        let bundle = check_promises(&r).unwrap();
        for (metric, c) in bundle.comparisons.iter().enumerate() {
            assert_eq!(
                (c.plan.assigned, c.plan.estimate_totals[metric]),
                exact(&r, metric)
            );
            for pin in &r.snapshot.commitments {
                let j = r
                    .snapshot
                    .jobs
                    .iter()
                    .position(|j| j.id == pin.job_id)
                    .unwrap();
                assert_eq!(c.plan.assignments[j].as_ref(), Some(&pin.resource_id));
            }
        }
    }
}

#[test]
fn validation_budget_is_not_infeasibility_and_replay_rejects_tampering() {
    let mut r = request();
    let full = check_promises(&r).unwrap();
    r.verification_work = 0;
    let limited = check_promises(&r).unwrap();
    for (a, b) in limited.comparisons.iter().zip(&full.comparisons) {
        assert_eq!(a.plan, b.plan);
        assert_eq!(a.evidence, PlanEvidence::VerifiedFeasibleOptimalityUnknown);
        assert_eq!(a.verification, VerificationOutcome::WorkLimit);
    }
    replay_decision(&limited).unwrap();
    let mut tampered = full.clone();
    tampered.comparisons[0].plan.assignments[0] = Some("crew-b".into());
    assert_eq!(
        replay_decision(&tampered),
        Err(PromiseError::ReplayMismatch)
    );
    tampered = full;
    tampered.comparisons[0].estimate_deltas[0] -= 1;
    assert_eq!(
        replay_decision(&tampered),
        Err(PromiseError::ReplayMismatch)
    );
}

#[test]
fn malformed_snapshot_and_conflicting_commitments_fail_closed() {
    let mut r = request();
    r.version = 2;
    assert_eq!(check_promises(&r), Err(PromiseError::UnsupportedVersion));
    let mut r = request();
    r.snapshot.commitments[0].resource_id = "crew-b".into();
    assert_eq!(check_promises(&r), Err(PromiseError::CommitmentConflict));
    let mut r = request();
    r.snapshot.resources[0].capacity = 1;
    assert!(matches!(
        check_promises(&r),
        Err(PromiseError::InvalidIncumbent(_))
    ));
    let mut r = request();
    r.snapshot.jobs[0].options[0].estimates.pop();
    assert_eq!(check_promises(&r), Err(PromiseError::WrongEstimateCount));
    let mut r = request();
    r.policies[0].minimize_metric = "invented-travel".into();
    assert_eq!(check_promises(&r), Err(PromiseError::InvalidReference));
    let mut r = request();
    r.snapshot.jobs[1].id = r.snapshot.jobs[0].id.clone();
    assert_eq!(check_promises(&r), Err(PromiseError::InvalidIdentity));
    let mut r = request();
    r.verification_work = 20_000_001;
    assert_eq!(check_promises(&r), Err(PromiseError::LimitExceeded));
    let mut r = request();
    r.snapshot.jobs = vec![r.snapshot.jobs[0].clone(); 201];
    assert_eq!(check_promises(&r), Err(PromiseError::LimitExceeded));
}

#[test]
fn all_fixed_and_no_eligible_work_are_honest_edge_cases() {
    let mut r = request();
    r.snapshot.jobs.truncate(1);
    r.snapshot.incumbent.truncate(1);
    r.verification_work = 1000;
    let b = check_promises(&r).unwrap();
    assert!(
        b.comparisons
            .iter()
            .all(|c| c.changes.is_empty()
                && c.evidence == PlanEvidence::ProvenOptimalUnderCommitments)
    );
    r.snapshot.commitments.clear();
    r.snapshot.incumbent[0] = None;
    r.snapshot.jobs[0].options.clear();
    r.verification_work = 1000;
    let b = check_promises(&r).unwrap();
    assert_eq!(b.comparisons[0].plan.assigned, 0);
    assert_eq!(b.comparisons[0].unassigned_job_ids, ["inspection-101"]);
    assert_eq!(
        b.comparisons[0].evidence,
        PlanEvidence::ProvenOptimalUnderCommitments
    );
}

#[test]
fn renamed_vocabulary_preserves_the_same_plan_and_estimates() {
    let r = request();
    let original = check_promises(&r).unwrap();
    let mut renamed = r;
    for (i, metric) in renamed.snapshot.metrics.iter_mut().enumerate() {
        metric.id = format!("metric-{i}");
    }
    for (i, policy) in renamed.policies.iter_mut().enumerate() {
        policy.id = format!("policy-{i}");
        policy.minimize_metric = format!("metric-{i}");
    }
    for i in 0..renamed.snapshot.resources.len() {
        let old = renamed.snapshot.resources[i].id.clone();
        let new = format!("resource-{i}");
        renamed.snapshot.resources[i].id = new.clone();
        for job in &mut renamed.snapshot.jobs {
            for option in &mut job.options {
                if option.resource_id == old {
                    option.resource_id = new.clone();
                }
            }
        }
        for a in renamed.snapshot.incumbent.iter_mut().flatten() {
            if *a == old {
                *a = new.clone();
            }
        }
        for pin in &mut renamed.snapshot.commitments {
            if pin.resource_id == old {
                pin.resource_id = new.clone();
            }
        }
    }
    for (i, job) in renamed.snapshot.jobs.iter_mut().enumerate() {
        for pin in &mut renamed.snapshot.commitments {
            if pin.job_id == job.id {
                pin.job_id = format!("item-{i}");
            }
        }
        job.id = format!("item-{i}");
    }
    let result = check_promises(&renamed).unwrap();
    for (a, b) in original.comparisons.iter().zip(&result.comparisons) {
        assert_eq!(a.plan.estimate_totals, b.plan.estimate_totals);
        assert_eq!(a.evidence, b.evidence);
        let original_indices: Vec<_> = a
            .plan
            .assignments
            .iter()
            .map(|id| {
                id.as_ref().map(|id| {
                    original
                        .request
                        .snapshot
                        .resources
                        .iter()
                        .position(|r| r.id == *id)
                        .unwrap()
                })
            })
            .collect();
        let renamed_indices: Vec<_> = b
            .plan
            .assignments
            .iter()
            .map(|id| {
                id.as_ref().map(|id| {
                    renamed
                        .snapshot
                        .resources
                        .iter()
                        .position(|r| r.id == *id)
                        .unwrap()
                })
            })
            .collect();
        assert_eq!(original_indices, renamed_indices);
    }
}

#[test]
fn backlog_keeps_six_confirmed_jobs_and_reports_measured_plan_differences() {
    let r: PromiseRequest =
        serde_json::from_str(include_str!("../../fixtures/promise-check/backlog.json")).unwrap();
    let b = check_promises(&r).unwrap();
    assert_eq!(b.incumbent.assigned, 41);
    assert_eq!(b.incumbent.estimate_totals, [415400, 1812]);
    assert_eq!(b.comparisons[0].plan.estimate_totals, [362000, 2126]);
    assert_eq!(b.comparisons[1].plan.estimate_totals, [450200, 1499]);
    for c in &b.comparisons {
        assert_eq!(c.plan.assigned, 42);
        assert_eq!(c.evidence, PlanEvidence::ProvenOptimalUnderCommitments);
        for pin in &r.snapshot.commitments {
            let j = r
                .snapshot
                .jobs
                .iter()
                .position(|j| j.id == pin.job_id)
                .unwrap();
            assert_eq!(c.plan.assignments[j].as_ref(), Some(&pin.resource_id));
        }
    }
    replay_decision(&b).unwrap();
}
