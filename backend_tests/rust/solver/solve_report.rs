use orchestrata_solver::*;

fn preference() -> Problem {
    Problem {
        variables: vec![Variable {
            name: "choice".into(),
            domain: vec![0, 1],
        }],
        constraints: vec![Constraint::PreferValue {
            variable: 0,
            value: 1,
            soft_weight: 7,
        }],
    }
}

#[test]
fn budget_and_proof_are_distinct_and_legacy_bytes_stay_stable() {
    let p = preference();
    let zero = solve_bounded(&p, 0).unwrap();
    assert_eq!(zero.termination(), SolveTermination::EvaluationLimit);
    assert_eq!(zero.evidence(), &SolutionEvidence::Unknown);
    assert_eq!(zero.verification_scope(), VerificationScope::NoIncumbent);
    let partial = solve_bounded(&p, 1).unwrap();
    assert_eq!(partial.termination(), SolveTermination::EvaluationLimit);
    let SolutionEvidence::VerifiedFeasible(s) = partial.evidence() else {
        panic!()
    };
    assert_eq!(s.score.soft, -7);
    verify_solution(&p, s).unwrap();
    assert_eq!(partial.proven_optimal_soft_score(), None);
    assert_eq!(
        partial.verification_scope(),
        VerificationScope::IndependentFeasibilityAndScore
    );
    assert_eq!(
        partial.objective(),
        SolveObjective::MaximizeSoftScoreSubjectToHardConstraints
    );
    let complete = solve_bounded(&p, 2).unwrap();
    assert_eq!(complete.termination(), SolveTermination::SearchExhausted);
    assert_eq!(complete.proven_optimal_soft_score(), Some(0));
    assert_eq!(
        complete.verification_scope(),
        VerificationScope::IndependentFeasibilityAndScoreWithExhaustiveSearch
    );
    let SolutionEvidence::ProvenOptimal(s) = complete.evidence() else {
        panic!()
    };
    assert_eq!(solve(&p, 2).unwrap().as_ref(), Some(s));
    assert_eq!(solve(&p, 1), Err(Error::TooManyAssignments));
    assert_eq!(
        serde_json::to_string(s).unwrap(),
        r#"{"values":[1],"score":{"hard":0,"soft":0},"evaluated":2}"#
    );
}

#[test]
fn no_incumbent_is_not_infeasibility_until_exhaustion() {
    let mut p = preference();
    p.constraints.push(Constraint::CountValue {
        variables: vec![0],
        value: 2,
        min: 1,
        max: 1,
        weight: 1,
        tier: CountValueTier::Hard,
    });
    assert_eq!(
        solve_bounded(&p, 1).unwrap().evidence(),
        &SolutionEvidence::Unknown
    );
    let done = solve_bounded(&p, 2).unwrap();
    assert_eq!(done.evidence(), &SolutionEvidence::ProvenInfeasible);
    assert_eq!(done.termination(), SolveTermination::SearchExhausted);
    assert_eq!(
        done.verification_scope(),
        VerificationScope::ExhaustiveInfeasibilitySearch
    );
    assert_eq!(done.proven_optimal_soft_score(), None);
    assert_eq!(solve(&p, 2), Ok(None));
}

#[test]
fn errors_empty_space_and_product_overflow_are_explicit() {
    let mut p = preference();
    p.variables[0].domain.clear();
    assert_eq!(solve_bounded(&p, 0), Err(Error::EmptyDomain(0)));
    let empty = Problem {
        variables: vec![],
        constraints: vec![],
    };
    assert_eq!(
        solve_bounded(&empty, 0).unwrap().evidence(),
        &SolutionEvidence::Unknown
    );
    assert!(matches!(
        solve_bounded(&empty, 1).unwrap().evidence(),
        SolutionEvidence::ProvenOptimal(_)
    ));
    let large = Problem {
        variables: vec![
            Variable {
                name: "x".into(),
                domain: vec![0, 1]
            };
            65
        ],
        constraints: vec![],
    };
    assert_eq!(solve(&large, 1), Err(Error::TooManyAssignments));
    let partial = solve_bounded(&large, 1).unwrap();
    assert_eq!(partial.evaluated(), 1);
    assert!(matches!(
        partial.evidence(),
        SolutionEvidence::VerifiedFeasible(_)
    ));
    let mut overflow = preference();
    overflow.variables[0].domain = vec![0];
    overflow.constraints = vec![
        Constraint::PreferValue {
            variable: 0,
            value: 1,
            soft_weight: i64::MAX
        };
        3
    ];
    assert_eq!(solve_bounded(&overflow, 1), Err(Error::ScoreOverflow));
    assert_eq!(solve(&overflow, 1), Err(Error::ScoreOverflow));
}

#[test]
fn every_prefix_matches_independent_small_instance_oracle() {
    for seed in 0..128usize {
        let target = (seed % 3) as i64;
        let weight = (seed % 7 + 1) as i64;
        let distinct = seed % 2 == 0;
        let required = seed % 3;
        let mut p = Problem {
            variables: (0..2)
                .map(|i| Variable {
                    name: format!("v{i}"),
                    domain: vec![0, 1, 2],
                })
                .collect(),
            constraints: vec![
                Constraint::PreferValue {
                    variable: 0,
                    value: target,
                    soft_weight: weight,
                },
                Constraint::CountValue {
                    variables: vec![0, 1],
                    value: 1,
                    min: required,
                    max: required,
                    weight: 1,
                    tier: CountValueTier::Hard,
                },
            ],
        };
        if distinct {
            p.constraints.push(Constraint::AllDifferent {
                variables: vec![0, 1],
                hard_weight: 1,
            });
        }
        for limit in 0..=10usize {
            let mut best: Option<(Vec<i64>, i64)> = None;
            for n in 0..limit.min(9) {
                let values = vec![(n / 3) as i64, (n % 3) as i64];
                if (distinct && values[0] == values[1])
                    || values.iter().filter(|&&v| v == 1).count() != required
                {
                    continue;
                }
                let soft = if values[0] == target { 0 } else { -weight };
                if best.as_ref().is_none_or(|(_, score)| soft > *score) {
                    best = Some((values, soft));
                }
            }
            let r = solve_bounded(&p, limit as u64).unwrap();
            assert_eq!(r.evaluated(), limit.min(9) as u64);
            assert_eq!(
                r.termination(),
                if limit >= 9 {
                    SolveTermination::SearchExhausted
                } else {
                    SolveTermination::EvaluationLimit
                }
            );
            match (best, r.evidence()) {
                (None, SolutionEvidence::Unknown) => assert!(limit < 9),
                (None, SolutionEvidence::ProvenInfeasible) => assert!(limit >= 9),
                (Some((values, soft)), SolutionEvidence::VerifiedFeasible(s)) => {
                    assert!(limit < 9);
                    assert_eq!((&s.values, s.score.soft), (&values, soft));
                    verify_solution(&p, s).unwrap();
                }
                (Some((values, soft)), SolutionEvidence::ProvenOptimal(s)) => {
                    assert!(limit >= 9);
                    assert_eq!((&s.values, s.score.soft), (&values, soft));
                    assert_eq!(solve(&p, limit as u64).unwrap().as_ref(), Some(s));
                }
                _ => panic!("incorrect evidence for seed {seed}, limit {limit}: {r:?}"),
            }
        }
    }
}
