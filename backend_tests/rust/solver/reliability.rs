use orchestrata_solver::{
    Constraint, Error, Problem, Score, Solution, SolutionValidationError, Variable, solve,
    verify_solution,
};

fn variable(name: &str, domain: &[i64]) -> Variable {
    Variable {
        name: name.to_owned(),
        domain: domain.to_vec(),
    }
}

fn dispatch_problem() -> Problem {
    // Domains encode technicians qualified and available for each overlapping job.
    Problem {
        variables: vec![
            variable("lift_repair", &[8, 3]),
            variable("electrical_inspection", &[8, 2]),
            variable("preventive_maintenance", &[3, 2]),
        ],
        constraints: vec![
            Constraint::AllDifferent {
                variables: vec![0, 1, 2],
                hard_weight: 1,
            },
            Constraint::PreferValue {
                variable: 0,
                value: 8,
                soft_weight: 5,
            },
            Constraint::PreferValue {
                variable: 1,
                value: 8,
                soft_weight: 3,
            },
        ],
    }
}

fn exhaustive_oracle(problem: &Problem) -> Option<(Vec<i64>, Score)> {
    let mut assignments = vec![Vec::new()];
    for variable in &problem.variables {
        assignments = assignments
            .into_iter()
            .flat_map(|prefix| {
                variable.domain.iter().map(move |value| {
                    let mut next = prefix.clone();
                    next.push(*value);
                    next
                })
            })
            .collect();
    }

    let mut best = None;
    for values in assignments {
        let Some(soft) = feasible_soft_score(problem, &values) else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(_, score): &(Vec<i64>, Score)| soft > score.soft)
        {
            best = Some((values, Score { hard: 0, soft }));
        }
    }
    best
}

fn feasible_soft_score(problem: &Problem, values: &[i64]) -> Option<i64> {
    let mut soft = 0;
    for constraint in &problem.constraints {
        match constraint {
            Constraint::AllDifferent { variables, .. } => {
                let distinct = variables
                    .iter()
                    .map(|&index| values[index])
                    .collect::<std::collections::BTreeSet<_>>();
                if distinct.len() < variables.len() {
                    return None;
                }
            }
            Constraint::NotEqualOffset {
                left,
                right,
                offset,
                ..
            } => {
                if i128::from(values[*left]) == i128::from(values[*right]) + i128::from(*offset) {
                    return None;
                }
            }
            Constraint::PreferValue {
                variable,
                value,
                soft_weight,
            } => {
                soft -= i64::from(values[*variable] != *value) * *soft_weight;
            }
        }
    }
    Some(soft)
}

#[test]
fn assigns_only_eligible_technicians_to_overlapping_jobs() {
    let problem = dispatch_problem();

    let result = solve(&problem, 8).unwrap().unwrap();

    assert_eq!(result.values, [8, 2, 3]);
    assert_eq!(result.score, Score { hard: 0, soft: -3 });
    assert_eq!(result.evaluated, 8);
    assert_eq!(verify_solution(&problem, &result), Ok(()));
}

#[test]
fn limits_do_not_claim_infeasibility_and_exact_budget_is_enough_to_prove_it() {
    let mut problem = dispatch_problem();
    problem.variables[1].domain = vec![8];
    problem.variables[0].domain = vec![8];

    assert_eq!(solve(&problem, 0), Err(Error::TooManyAssignments));
    assert_eq!(solve(&problem, 1), Err(Error::TooManyAssignments));
    assert_eq!(solve(&problem, 2), Ok(None));
}

#[test]
fn search_space_overflow_is_reported_without_enumeration() {
    let problem = Problem {
        variables: (0..65)
            .map(|index| variable(&format!("job_{index}"), &[0, 1]))
            .collect(),
        constraints: vec![],
    };

    assert_eq!(solve(&problem, u64::MAX), Err(Error::TooManyAssignments));
}

#[test]
fn tiny_problems_match_an_independent_exhaustive_oracle() {
    for first_domain in [&[5, -2][..], &[-2, 5], &[5], &[-2]] {
        for second_domain in [&[5, -2][..], &[-2, 5], &[5], &[-2]] {
            for offset in [-7, 0, 7] {
                for preferred in [-2, 5, 9] {
                    let problem = Problem {
                        variables: vec![
                            variable("first", first_domain),
                            variable("second", second_domain),
                        ],
                        constraints: vec![
                            Constraint::AllDifferent {
                                variables: vec![0, 1],
                                hard_weight: 4,
                            },
                            Constraint::NotEqualOffset {
                                left: 0,
                                right: 1,
                                offset,
                                hard_weight: 1,
                            },
                            Constraint::PreferValue {
                                variable: 0,
                                value: preferred,
                                soft_weight: 2,
                            },
                        ],
                    };
                    let result = solve(&problem, 4).unwrap();
                    if let Some(solution) = &result {
                        assert_eq!(verify_solution(&problem, solution), Ok(()));
                    }
                    assert_eq!(
                        result.map(|solution| (solution.values, solution.score)),
                        exhaustive_oracle(&problem),
                        "domains {first_domain:?} and {second_domain:?}, offset {offset}, preference {preferred}"
                    );
                }
            }
        }
    }
}

#[test]
fn verifier_rejects_malformed_and_forged_results() {
    let problem = dispatch_problem();
    let solution = solve(&problem, 8).unwrap().unwrap();

    let wrong_length = Solution {
        values: vec![8, 2],
        ..solution.clone()
    };
    assert_eq!(
        verify_solution(&problem, &wrong_length),
        Err(SolutionValidationError::WrongValueCount)
    );

    let outsider = Solution {
        values: vec![8, 2, 9],
        ..solution.clone()
    };
    assert_eq!(
        verify_solution(&problem, &outsider),
        Err(SolutionValidationError::ValueOutsideDomain(2))
    );

    let double_booked = Solution {
        values: vec![8, 8, 3],
        ..solution.clone()
    };
    assert_eq!(
        verify_solution(&problem, &double_booked),
        Err(SolutionValidationError::HardConstraintViolation(0))
    );

    let spoofed_score = Solution {
        score: Score { hard: 0, soft: 0 },
        ..solution
    };
    assert_eq!(
        verify_solution(&problem, &spoofed_score),
        Err(SolutionValidationError::ScoreMismatch)
    );
}

#[test]
fn declared_domain_order_breaks_ties() {
    let problem = Problem {
        variables: vec![variable("job", &[8, 2])],
        constraints: vec![],
    };

    assert_eq!(solve(&problem, 2).unwrap().unwrap().values, [8]);
}

#[test]
fn minimum_representable_soft_score_remains_valid() {
    let problem = Problem {
        variables: vec![variable("job", &[0])],
        constraints: vec![
            Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight: i64::MAX,
            },
            Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight: 1,
            },
        ],
    };
    let solution = solve(&problem, 1).unwrap().unwrap();

    assert_eq!(
        solution.score,
        Score {
            hard: 0,
            soft: i64::MIN
        }
    );
    assert_eq!(verify_solution(&problem, &solution), Ok(()));
}

fn dominated_overflow_problem() -> Problem {
    Problem {
        variables: vec![variable("job", &[0, 1])],
        constraints: [i64::MAX, 2]
            .into_iter()
            .map(|soft_weight| Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight,
            })
            .collect(),
    }
}

#[test]
fn infeasible_candidate_with_overflowing_hard_score_does_not_abort_search() {
    let problem = Problem {
        variables: vec![variable("first_job", &[0, 1]), variable("second_job", &[0])],
        constraints: [i64::MAX, 2]
            .into_iter()
            .map(|hard_weight| Constraint::AllDifferent {
                variables: vec![0, 1],
                hard_weight,
            })
            .collect(),
    };

    let best = solve(&problem, 2).unwrap().unwrap();

    assert_eq!(best.values, [1, 0]);
    assert_eq!(best.score, Score { hard: 0, soft: 0 });
}

#[test]
fn dominated_feasible_candidate_with_overflowing_soft_score_does_not_abort_search() {
    let problem = dominated_overflow_problem();

    let best = solve(&problem, 2).unwrap().unwrap();

    assert_eq!(best.values, [1]);
    assert_eq!(best.score, Score { hard: 0, soft: 0 });
}
