use std::{ffi::CStr, ptr};

use orchestrata_ffi::{Status, orchestrata_free_json_v1, orchestrata_solve_json_v1};
use orchestrata_solver::{
    CompiledProblem, Constraint, Error, IncrementalScorer, Problem, ResourceCapacityError,
    ResourceLimit, ResourceLoad, Score, SearchOutcome, SearchStop, Solution,
    SolutionValidationError, Variable, score, search_locally, solve, validate, verify_solution,
};

fn variable(name: &str, domain: &[i64]) -> Variable {
    Variable {
        name: name.into(),
        domain: domain.to_vec(),
    }
}

const fn item(variable: usize, load: u64) -> ResourceLoad {
    ResourceLoad { variable, load }
}

const fn limit(resource: i64, capacity: u64) -> ResourceLimit {
    ResourceLimit { resource, capacity }
}

fn capacity(items: Vec<ResourceLoad>, limits: Vec<ResourceLimit>) -> Constraint {
    Constraint::ResourceCapacity { items, limits }
}

fn cargo_fixture() -> Problem {
    Problem {
        variables: vec![
            variable("crate-a", &[-10, 20]),
            variable("crate-b", &[-10, 20]),
            variable("crate-c", &[-10]),
        ],
        constraints: vec![
            capacity(
                vec![item(0, 6), item(1, 5), item(2, 2)],
                vec![limit(-10, 8), limit(20, 7)],
            ),
            Constraint::PreferValue {
                variable: 0,
                value: -10,
                soft_weight: 3,
            },
        ],
    }
}

#[test]
fn exact_search_uses_per_resource_capacity_and_reports_budget_separately() {
    let problem = cargo_fixture();
    let expected = [
        ([-10, -10, -10], Score { hard: -5, soft: 0 }),
        ([-10, 20, -10], Score { hard: 0, soft: 0 }),
        ([20, -10, -10], Score { hard: 0, soft: -3 }),
        ([20, 20, -10], Score { hard: -4, soft: -3 }),
    ];
    for (values, grade) in expected {
        assert_eq!(score(&problem, &values), Ok(grade));
    }

    assert_eq!(solve(&problem, 3), Err(Error::TooManyAssignments));
    let result = solve(&problem, 4).unwrap().unwrap();
    assert_eq!(result.values, [-10, 20, -10]);
    verify_solution(&problem, &result).unwrap();

    let forged = Solution {
        values: vec![-10, -10, -10],
        ..result
    };
    assert_eq!(
        verify_solution(&problem, &forged),
        Err(SolutionValidationError::HardConstraintViolation(0))
    );
}

#[test]
fn capacity_scopes_are_independent_and_zero_load_is_valid() {
    let problem = Problem {
        variables: vec![variable("a", &[9, 11]), variable("b", &[9, 11])],
        constraints: vec![
            capacity(
                vec![item(0, 6), item(1, 7)],
                vec![limit(9, 6), limit(11, 7)],
            ),
            capacity(
                vec![item(0, 0), item(1, 3)],
                vec![limit(9, 0), limit(11, 3)],
            ),
        ],
    };
    let result = solve(&problem, 4).unwrap().unwrap();
    assert_eq!(result.values, [9, 11]);
    verify_solution(&problem, &result).unwrap();
}

#[test]
fn exact_infeasibility_and_local_search_unknown_are_not_conflated() {
    let problem = Problem {
        variables: vec![variable("a", &[3]), variable("b", &[3])],
        constraints: vec![capacity(vec![item(0, 3), item(1, 5)], vec![limit(3, 7)])],
    };
    assert_eq!(solve(&problem, 1), Ok(None));
    assert_eq!(
        search_locally(&problem, 1),
        Ok(SearchOutcome::NoFeasibleIncumbent {
            evaluated: 1,
            stop: SearchStop::NeighborhoodExhausted
        })
    );
}

#[test]
fn compiled_scores_match_full_scoring_on_move_commit_and_rollback() {
    let problem = cargo_fixture();
    let compiled = CompiledProblem::new(&problem).unwrap();
    assert_eq!(compiled.incident_constraints(0), Some(&[0, 1][..]));
    assert_eq!(compiled.incident_constraints(1), Some(&[0][..]));
    assert_eq!(compiled.incident_constraints(2), Some(&[0][..]));

    let mut scorer = IncrementalScorer::new(&compiled, &[-10, -10, -10]).unwrap();
    for cycle in 0..8 {
        let variable = cycle % 2;
        let original = scorer.values().to_vec();
        let value = if original[variable] == -10 { 20 } else { -10 };
        let trial = scorer.try_move(variable, value).unwrap();
        let expected = score(&problem, trial.values());
        assert_eq!(trial.score(), expected);
        assert_eq!(trial.is_feasible(), expected.unwrap().hard == 0);
        if cycle % 3 == 0 {
            drop(trial);
            assert_eq!(scorer.values(), original);
        } else {
            trial.commit();
            assert_eq!(scorer.values()[variable], value);
        }
        assert_eq!(scorer.score(), score(&problem, scorer.values()));
    }

    let SearchOutcome::FeasibleIncumbent { solution, .. } = search_locally(&problem, 12).unwrap()
    else {
        panic!("expected a feasible local incumbent")
    };
    verify_solution(&problem, &solution).unwrap();
}

#[test]
fn rejects_ambiguous_scope_and_unsafe_worst_case_per_resource() {
    use ResourceCapacityError::*;
    let variables = vec![variable("a", &[1, 2]), variable("b", &[2])];
    let cases = [
        (vec![], vec![], EmptyItems),
        (
            vec![item(0, 2), item(0, 1)],
            vec![limit(1, 3), limit(2, 3)],
            DuplicateItem(0),
        ),
        (
            vec![item(0, 2)],
            vec![limit(1, 3), limit(1, 3)],
            DuplicateLimit(1),
        ),
        (vec![item(0, 2)], vec![limit(1, 3)], MissingLimit(2)),
        (
            vec![item(0, 2)],
            vec![limit(1, 3), limit(2, 3), limit(3, 3)],
            UnusedLimit(3),
        ),
        (
            vec![item(0, u64::MAX), item(1, 1)],
            vec![limit(1, u64::MAX), limit(2, u64::MAX)],
            PotentialLoadOverflow(2),
        ),
    ];
    for (items, limits, invalid) in cases {
        let problem = Problem {
            variables: variables.clone(),
            constraints: vec![capacity(items, limits)],
        };
        let expected = Err(Error::InvalidResourceCapacity(invalid));
        assert_eq!(validate(&problem), expected);
        assert_eq!(solve(&problem, 10).map(|_| ()), expected);
    }

    let problem = Problem {
        variables: vec![variable("a", &[1, 1])],
        constraints: vec![capacity(vec![item(0, 1)], vec![limit(1, 3)])],
    };
    assert_eq!(
        validate(&problem),
        Err(Error::InvalidResourceCapacity(DuplicateDomainValue {
            variable: 0,
            resource: 1
        }))
    );
    let invalid_index = Problem {
        variables,
        constraints: vec![capacity(vec![item(9, 0)], vec![limit(1, 1)])],
    };
    assert_eq!(validate(&invalid_index), Err(Error::InvalidVariable(9)));
}

#[test]
fn supports_full_unsigned_loads_and_does_not_confuse_score_overflow_with_infeasibility() {
    let disjoint = Problem {
        variables: vec![variable("a", &[1]), variable("b", &[2])],
        constraints: vec![capacity(
            vec![item(0, u64::MAX), item(1, u64::MAX)],
            vec![limit(1, u64::MAX), limit(2, u64::MAX)],
        )],
    };
    assert!(solve(&disjoint, 1).unwrap().is_some());

    for (load, expected) in [
        (
            1_u64 << 63,
            Ok(Score {
                hard: i64::MIN,
                soft: 0,
            }),
        ),
        ((1_u64 << 63) + 1, Err(Error::ScoreOverflow)),
    ] {
        let problem = Problem {
            variables: vec![variable("heavy", &[1])],
            constraints: vec![capacity(vec![item(0, load)], vec![limit(1, 0)])],
        };
        let compiled = CompiledProblem::new(&problem).unwrap();
        let scorer = IncrementalScorer::new(&compiled, &[1]).unwrap();
        assert_eq!(scorer.score(), expected);
        assert_eq!(score(&problem, &[1]), expected);
        assert!(!scorer.is_feasible());
        assert_eq!(solve(&problem, 1), Ok(None));
    }
}

#[test]
fn additive_json_constraint_preserves_the_native_abi_contract() {
    let input = br#"{"variables":[{"name":"box","domain":[-2]}],"constraints":[{"kind":"resource_capacity","items":[{"variable":0,"load":18446744073709551615}],"limits":[{"resource":-2,"capacity":18446744073709551615}]}]}"#;
    let problem: Problem = serde_json::from_slice(input).unwrap();
    assert_eq!(
        serde_json::from_slice::<Problem>(&serde_json::to_vec(&problem).unwrap()).unwrap(),
        problem
    );
    let mut result = ptr::null_mut();
    // SAFETY: the request and output slot are live, aligned, and disjoint.
    let status = unsafe { orchestrata_solve_json_v1(input.as_ptr(), input.len(), 1, &mut result) };
    assert_eq!(status, Status::Ok as i32);
    // SAFETY: success returns a live NUL-terminated buffer owned by the caller.
    let output = unsafe { CStr::from_ptr(result).to_bytes().to_vec() };
    // SAFETY: result is an unmodified native output released exactly once.
    unsafe {
        orchestrata_free_json_v1(result);
    }
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output).unwrap()["solution"]["values"],
        serde_json::json!([-2])
    );

    let invalid = br#"{"variables":[{"name":"box","domain":[-2]}],"constraints":[{"kind":"resource_capacity","items":[{"variable":0,"load":1}],"limits":[]}]}"#;
    let mut result = ptr::null_mut();
    // SAFETY: the request and output slot are live, aligned, and disjoint.
    let status =
        unsafe { orchestrata_solve_json_v1(invalid.as_ptr(), invalid.len(), 1, &mut result) };
    assert_eq!(status, Status::InvalidProblem as i32);
    assert!(result.is_null());

    let negative = String::from_utf8(invalid.to_vec())
        .unwrap()
        .replace("\"load\":1", "\"load\":-1");
    assert!(serde_json::from_str::<Problem>(&negative).is_err());
}

#[test]
fn dispatch_requires_both_minimum_job_coverage_and_weighted_vehicle_limits() {
    use orchestrata_solver::CountValueTier;

    // Each job is assigned to a dispatch team. Team 10 must handle two jobs,
    // but can carry only five units. Team 20 can carry six units.
    let model = Problem {
        variables: vec![
            variable("heavy_delivery", &[10, 20]),
            variable("medium_delivery", &[10, 20]),
            variable("light_delivery", &[10, 20]),
        ],
        constraints: vec![
            Constraint::CountValue {
                variables: vec![0, 1, 2],
                value: 10,
                min: 2,
                max: 3,
                weight: 7,
                tier: CountValueTier::Hard,
            },
            capacity(
                vec![item(0, 4), item(1, 3), item(2, 2)],
                vec![limit(10, 5), limit(20, 6)],
            ),
        ],
    };
    assert_eq!(score(&model, &[10, 20, 20]).unwrap().hard, -7);
    assert_eq!(score(&model, &[10, 10, 20]).unwrap().hard, -2);
    let result = solve(&model, 8).unwrap().unwrap();
    assert_eq!(result.values, [20, 10, 10]);
    verify_solution(&model, &result).unwrap();

    let compiled = CompiledProblem::new(&model).unwrap();
    assert_eq!(compiled.incident_constraints(0), Some(&[0, 1][..]));
    let mut scorer = IncrementalScorer::new(&compiled, &[10, 10, 20]).unwrap();
    scorer.try_move(0, 20).unwrap().commit();
    let trial = scorer.try_move(2, 10).unwrap();
    assert_eq!(trial.score(), score(&model, &result.values));
    trial.commit();
    assert_eq!(scorer.values(), result.values);
    assert!(scorer.is_feasible());

    let mut constrained = model.clone();
    constrained.constraints[1] = capacity(
        vec![item(0, 4), item(1, 3), item(2, 2)],
        vec![limit(10, 4), limit(20, 6)],
    );
    assert_eq!(solve(&constrained, 8).unwrap(), None);
}

#[test]
fn resource_scratch_matches_independent_assignment_and_score_oracles() {
    const LEFT: i64 = i64::MIN;
    const RIGHT: i64 = i64::MAX;
    for seed in 0..256_u64 {
        let first = [seed & 3, (seed >> 2) & 3, (seed >> 4) & 3];
        let second = [(seed >> 5) & 3, seed & 1, (seed >> 7) & 1];
        let bounds = [seed % 5, (seed / 7) % 7, (seed / 3) % 6, (seed / 11) % 7];
        let problem = Problem {
            variables: vec![
                variable("a", &[LEFT, RIGHT]),
                variable("b", &[RIGHT, LEFT]),
                variable("c", &[LEFT, RIGHT]),
            ],
            constraints: vec![
                capacity(
                    first.iter().enumerate().map(|(i, &n)| item(i, n)).collect(),
                    vec![limit(LEFT, bounds[0]), limit(RIGHT, bounds[1])],
                ),
                capacity(
                    second
                        .iter()
                        .enumerate()
                        .map(|(i, &n)| item(i, n))
                        .collect(),
                    vec![limit(RIGHT, bounds[3]), limit(LEFT, bounds[2])],
                ),
                Constraint::PreferValue {
                    variable: 1,
                    value: LEFT,
                    soft_weight: 2,
                },
            ],
        };
        let mut expected: Option<Solution> = None;
        for a in [LEFT, RIGHT] {
            for b in [RIGHT, LEFT] {
                for c in [LEFT, RIGHT] {
                    let values = [a, b, c];
                    let overload = |loads: [u64; 3], resource, cap: u64| {
                        values
                            .iter()
                            .zip(loads)
                            .filter(|(v, _)| **v == resource)
                            .map(|(_, n)| n)
                            .sum::<u64>()
                            .saturating_sub(cap)
                    };
                    let hard = -((overload(first, LEFT, bounds[0])
                        + overload(first, RIGHT, bounds[1])
                        + overload(second, LEFT, bounds[2])
                        + overload(second, RIGHT, bounds[3]))
                        as i64);
                    let grade = Score {
                        hard,
                        soft: if b == LEFT { 0 } else { -2 },
                    };
                    assert_eq!(score(&problem, &values), Ok(grade));
                    if hard == 0 && expected.as_ref().is_none_or(|s| s.score.soft < grade.soft) {
                        expected = Some(Solution {
                            values: values.to_vec(),
                            score: grade,
                            evaluated: 8,
                        });
                    }
                }
            }
        }
        assert_eq!(solve(&problem, 7), Err(Error::TooManyAssignments));
        assert_eq!(solve(&problem, 8), Ok(expected), "seed {seed}");
    }
}

#[test]
fn public_verification_checks_invalid_input_before_claimed_assignment() {
    let invalid = Problem {
        variables: vec![variable("box", &[1])],
        constraints: vec![capacity(vec![item(0, 1)], vec![])],
    };
    let forged = Solution {
        values: vec![],
        score: Score { hard: 0, soft: 0 },
        evaluated: 1,
    };
    assert_eq!(
        verify_solution(&invalid, &forged),
        Err(SolutionValidationError::InvalidProblem(
            Error::InvalidResourceCapacity(ResourceCapacityError::MissingLimit(1))
        ))
    );
    assert_eq!(
        verify_solution(&cargo_fixture(), &forged),
        Err(SolutionValidationError::WrongValueCount)
    );
}

#[test]
fn validated_exact_path_still_returns_a_checked_solution() {
    let p = cargo_fixture();
    let raw = solve(&p, 4).unwrap().unwrap();
    let prepared = orchestrata_solver::PreparedProblem::prepare(p.clone())
        .unwrap()
        .solve(4)
        .unwrap()
        .unwrap();
    assert_eq!(raw, prepared);
    verify_solution(&p, &raw).unwrap();
    assert_eq!(solve(&p, 3), Err(Error::TooManyAssignments));
    assert_eq!(
        orchestrata_solver::PreparedProblem::prepare(p)
            .unwrap()
            .solve(3),
        Err(Error::TooManyAssignments)
    );
}
