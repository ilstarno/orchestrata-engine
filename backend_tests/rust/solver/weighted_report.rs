use orchestrata_solver::capacity_matching::weighted::*;

fn problem() -> WeightedMatchProblem {
    WeightedMatchProblem {
        worker_capacities: vec![1, 1],
        eligible_workers_by_job: vec![
            vec![
                WorkerCost { worker: 0, cost: 9 },
                WorkerCost { worker: 1, cost: 1 },
            ],
            vec![
                WorkerCost { worker: 0, cost: 1 },
                WorkerCost { worker: 1, cost: 9 },
            ],
        ],
    }
}

#[test]
fn solve_completion_is_separate_from_verification_budget() {
    let p = problem();
    let legacy = match_capacity_min_cost(&p).unwrap();
    for budget in [None, Some(0), Some(1), Some(10000)] {
        let r = solve_weighted_report(&p, budget).unwrap();
        assert_eq!(r.assignment(), &legacy);
        assert_eq!(r.termination(), WeightedTermination::Completed);
        assert_eq!(
            r.evidence(),
            if matches!(
                r.certificate_evaluation(),
                CertificateEvaluation::Verified(_)
            ) {
                WeightedEvidence::ProvenOptimal
            } else {
                WeightedEvidence::IndependentlyVerifiedFeasibleAndAccounted
            }
        );
        assert_eq!(
            r.objective(),
            WeightedObjective::MaximizeAssignedThenMinimizeSuppliedCost
        );
        match budget {
            None => assert_eq!(
                r.certificate_evaluation(),
                &CertificateEvaluation::NotRequested
            ),
            Some(0 | 1) => assert_eq!(
                r.certificate_evaluation(),
                &CertificateEvaluation::WorkLimit
            ),
            _ => assert!(matches!(
                r.certificate_evaluation(),
                CertificateEvaluation::Verified(_)
            )),
        }
    }
    let full = solve_weighted_report(&p, Some(10000)).unwrap();
    let CertificateEvaluation::Verified(certificate) = full.certificate_evaluation() else {
        panic!()
    };
    assert_eq!(
        solve_weighted_report(&p, Some(certificate.work)).unwrap(),
        full
    );
    assert_eq!(
        solve_weighted_report(&p, Some(certificate.work - 1))
            .unwrap()
            .certificate_evaluation(),
        &CertificateEvaluation::WorkLimit
    );
}

#[test]
fn feasible_improvements_and_forgery_have_distinct_results() {
    let p = problem();
    let bad_cost = WeightedMatchAssignment {
        workers_by_job: vec![Some(0), Some(1)],
        assigned: 2,
        total_cost: 18,
    };
    let r = assess_weighted_assignment(&p, bad_cost.clone(), Some(10000)).unwrap();
    assert_eq!(r.termination(), WeightedTermination::NotRun);
    assert_eq!(
        r.certificate_evaluation(),
        &CertificateEvaluation::Rejected(WeightedVerificationError::NotMinimumCost)
    );
    assert_eq!(
        r.evidence(),
        WeightedEvidence::IndependentlyVerifiedFeasibleAndAccounted
    );
    let partial = WeightedMatchAssignment {
        workers_by_job: vec![Some(1), None],
        assigned: 1,
        total_cost: 1,
    };
    assert_eq!(
        assess_weighted_assignment(&p, partial, Some(10000))
            .unwrap()
            .certificate_evaluation(),
        &CertificateEvaluation::Rejected(WeightedVerificationError::NotMaximumCoverage)
    );
    let mut forged = bad_cost.clone();
    forged.total_cost = 0;
    assert_eq!(
        assess_weighted_assignment(&p, forged, Some(0)),
        Err(WeightedVerificationError::CostMismatch)
    );
    let mut forged = bad_cost;
    forged.assigned = 0;
    assert_eq!(
        assess_weighted_assignment(&p, forged, None),
        Err(WeightedVerificationError::CountMismatch)
    );
}

#[test]
fn ties_and_empty_assignments_do_not_expand_evidence_scope() {
    let mut p = problem();
    for row in &mut p.eligible_workers_by_job {
        for edge in row {
            edge.cost = 1;
        }
    }
    for plan in [vec![Some(0), Some(1)], vec![Some(1), Some(0)]] {
        let r = assess_weighted_assignment(
            &p,
            WeightedMatchAssignment {
                workers_by_job: plan,
                assigned: 2,
                total_cost: 2,
            },
            Some(10000),
        )
        .unwrap();
        assert!(matches!(
            r.certificate_evaluation(),
            CertificateEvaluation::Verified(_)
        ));
        assert_eq!(r.evidence(), WeightedEvidence::ProvenOptimal);
    }
    let empty = WeightedMatchProblem {
        worker_capacities: vec![],
        eligible_workers_by_job: vec![],
    };
    let r = solve_weighted_report(&empty, Some(0)).unwrap();
    assert_eq!(r.assignment().assigned, 0);
    assert_eq!(
        r.certificate_evaluation(),
        &CertificateEvaluation::Verified(WeightedMatchCertificate {
            maximum_assigned: 0,
            minimum_cost: 0,
            work: 0
        })
    );
    assert_eq!(r.evidence(), WeightedEvidence::ProvenOptimal);
}
