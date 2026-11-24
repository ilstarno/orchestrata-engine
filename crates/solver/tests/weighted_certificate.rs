use orchestrata_solver::capacity_matching::weighted::*;

fn proposals(p: &WeightedMatchProblem) -> Vec<WeightedMatchAssignment> {
    let mut result = Vec::new();
    for code in 0..3usize.pow(p.eligible_workers_by_job.len() as u32) {
        let mut rest = code;
        let mut loads = [0; 2];
        let mut values = Vec::new();
        let mut count = 0;
        let mut cost = 0;
        let mut valid = true;
        for edges in &p.eligible_workers_by_job {
            let v = rest % 3;
            rest /= 3;
            if v == 2 {
                values.push(None);
                continue;
            }
            values.push(Some(v));
            loads[v] += 1;
            if let Some(e) = edges.iter().find(|e| e.worker == v) {
                cost += u64::from(e.cost);
                count += 1;
            } else {
                valid = false;
            }
        }
        if valid && loads.iter().zip(&p.worker_capacities).all(|(a, b)| a <= b) {
            result.push(WeightedMatchAssignment {
                workers_by_job: values,
                assigned: count,
                total_cost: cost,
            });
        }
    }
    result
}
#[test]
fn certificate_acceptance_matches_independent_exhaustive_oracle() {
    for seed in 0..256usize {
        let p = WeightedMatchProblem {
            worker_capacities: vec![seed % 3, (seed / 3) % 3],
            eligible_workers_by_job: (0..3)
                .map(|job| {
                    (0..2)
                        .filter(|worker| (seed >> (job * 2 + worker)) & 1 == 1)
                        .map(|worker| WorkerCost {
                            worker,
                            cost: ((seed + job * 7 + worker * 3) % 11) as u32,
                        })
                        .collect()
                })
                .collect(),
        };
        let all = proposals(&p);
        let best = all
            .iter()
            .map(|a| (a.assigned, std::cmp::Reverse(a.total_cost)))
            .max()
            .unwrap();
        for a in &all {
            let report = assess_weighted_assignment(&p, a.clone(), Some(1_000_000)).unwrap();
            assert_eq!(
                report.evidence() == WeightedEvidence::ProvenOptimal,
                (a.assigned, std::cmp::Reverse(a.total_cost)) == best,
                "report seed {seed}, {a:?}"
            );
            assert_eq!(report.termination(), WeightedTermination::NotRun);
            assert_eq!(
                verify_weighted_assignment(&p, a, 1_000_000).is_ok(),
                (a.assigned, std::cmp::Reverse(a.total_cost)) == best,
                "seed {seed}, {a:?}"
            );
        }
        let actual = match_capacity_min_cost(&p).unwrap();
        assert_eq!(
            (actual.assigned, std::cmp::Reverse(actual.total_cost)),
            best
        );
        assert!(verify_weighted_assignment(&p, &actual, 1_000_000).is_ok());
        let mut reordered = p.clone();
        for edges in &mut reordered.eligible_workers_by_job {
            edges.reverse();
        }
        assert!(verify_weighted_assignment(&reordered, &actual, 1_000_000).is_ok());
    }
}
#[test]
fn rejects_full_flow_cost_cycle_and_incomplete_coverage() {
    let p = WeightedMatchProblem {
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
    };
    let bad = WeightedMatchAssignment {
        workers_by_job: vec![Some(0), Some(1)],
        assigned: 2,
        total_cost: 18,
    };
    assert_eq!(
        verify_weighted_assignment(&p, &bad, 10000),
        Err(WeightedVerificationError::NotMinimumCost)
    );
    let partial = WeightedMatchAssignment {
        workers_by_job: vec![Some(1), None],
        assigned: 1,
        total_cost: 1,
    };
    assert_eq!(
        verify_weighted_assignment(&p, &partial, 10000),
        Err(WeightedVerificationError::NotMaximumCoverage)
    );
}
#[test]
fn budgets_and_accounting_fail_closed() {
    let p = WeightedMatchProblem {
        worker_capacities: vec![2],
        eligible_workers_by_job: vec![
            vec![WorkerCost {
                worker: 0,
                cost: u32::MAX
            }];
            2
        ],
    };
    let good = match_capacity_min_cost(&p).unwrap();
    let c = verify_weighted_assignment(&p, &good, 10000).unwrap();
    assert_eq!(c.minimum_cost, 2 * u64::from(u32::MAX));
    assert_eq!(verify_weighted_assignment(&p, &good, c.work), Ok(c.clone()));
    assert_eq!(
        verify_weighted_assignment(&p, &good, c.work - 1),
        Err(WeightedVerificationError::WorkLimit)
    );
    let mut bad = good.clone();
    bad.total_cost -= 1;
    assert_eq!(
        verify_weighted_assignment(&p, &bad, 10000),
        Err(WeightedVerificationError::CostMismatch)
    );
    bad = good.clone();
    bad.assigned -= 1;
    assert_eq!(
        verify_weighted_assignment(&p, &bad, 10000),
        Err(WeightedVerificationError::CountMismatch)
    );
    bad = good.clone();
    bad.workers_by_job[0] = Some(1);
    assert_eq!(
        verify_weighted_assignment(&p, &bad, 10000),
        Err(WeightedVerificationError::InvalidAssignment)
    );
    let mut capacity = p.clone();
    capacity.worker_capacities[0] = 1;
    assert_eq!(
        verify_weighted_assignment(&capacity, &good, 10000),
        Err(WeightedVerificationError::InvalidAssignment)
    );
}
#[test]
fn record_limit_empty_and_dense_cases() {
    let empty = WeightedMatchProblem {
        worker_capacities: vec![],
        eligible_workers_by_job: vec![],
    };
    assert!(
        verify_weighted_assignment(&empty, &match_capacity_min_cost(&empty).unwrap(), 0).is_ok()
    );
    let dense = WeightedMatchProblem {
        worker_capacities: vec![1; 200],
        eligible_workers_by_job: (0..200)
            .map(|j| {
                (0..200)
                    .map(|worker| WorkerCost {
                        worker,
                        cost: if j == worker { 0 } else { 1 },
                    })
                    .collect()
            })
            .collect(),
    };
    let assignment = WeightedMatchAssignment {
        workers_by_job: (0..200).map(Some).collect(),
        assigned: 200,
        total_cost: 0,
    };
    let c = verify_weighted_assignment(&dense, &assignment, 20_000_000).unwrap();
    assert_eq!(c.work, 40400);
    let mut large = dense;
    large.worker_capacities.push(1);
    assert!(matches!(
        verify_weighted_assignment(&large, &assignment, 20_000_000),
        Err(WeightedVerificationError::InvalidProblem(_))
    ));
}
