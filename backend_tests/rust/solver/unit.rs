use orchestrata_solver::*;

fn queens(n: usize) -> Problem {
    let variables = (0..n)
        .map(|i| Variable {
            name: format!("queen_{i}"),
            domain: (0..n as i64).collect(),
        })
        .collect();
    let mut constraints = vec![Constraint::AllDifferent {
        variables: (0..n).collect(),
        hard_weight: 1,
    }];
    for i in 0..n {
        for j in i + 1..n {
            let delta = j as i64 - i as i64;
            constraints.push(Constraint::NotEqualOffset {
                left: i,
                right: j,
                offset: delta,
                hard_weight: 1,
            });
            constraints.push(Constraint::NotEqualOffset {
                left: i,
                right: j,
                offset: -delta,
                hard_weight: 1,
            });
        }
    }
    Problem {
        variables,
        constraints,
    }
}

#[test]
fn four_queens_has_a_feasible_solution() {
    let problem = queens(4);
    let solution = solve(&problem, 256).unwrap().unwrap();
    assert_eq!(solution.score.hard, 0);
    assert_eq!(score(&problem, &solution.values).unwrap(), solution.score);
    assert_eq!(solution.evaluated, 256);
}

#[test]
fn impossible_three_queens_is_reported_infeasible() {
    assert_eq!(solve(&queens(3), 27), Ok(None));
}

#[test]
fn assignment_prefers_soft_objective_after_feasibility() {
    let problem = Problem {
        variables: vec![
            Variable {
                name: "a".into(),
                domain: vec![0, 1],
            },
            Variable {
                name: "b".into(),
                domain: vec![0, 1],
            },
        ],
        constraints: vec![
            Constraint::AllDifferent {
                variables: vec![0, 1],
                hard_weight: 1,
            },
            Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight: 3,
            },
        ],
    };
    let solution = solve(&problem, 4).unwrap().unwrap();
    assert_eq!(solution.values, vec![1, 0]);
    assert_eq!(solution.score, Score { hard: 0, soft: 0 });
}

#[test]
fn invalid_reference_is_rejected() {
    let problem = Problem {
        variables: vec![Variable {
            name: "a".into(),
            domain: vec![1],
        }],
        constraints: vec![Constraint::PreferValue {
            variable: 5,
            value: 1,
            soft_weight: 1,
        }],
    };
    assert_eq!(validate(&problem), Err(Error::InvalidVariable(5)));
}

#[test]
fn score_overflow_is_reported_instead_of_saturating() {
    let problem = Problem {
        variables: vec![Variable {
            name: "resource".into(),
            domain: vec![0],
        }],
        constraints: vec![
            Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight: i64::MAX,
            },
            Constraint::PreferValue {
                variable: 0,
                value: 2,
                soft_weight: 2,
            },
        ],
    };

    assert_eq!(score(&problem, &[0]), Err(Error::ScoreOverflow));
    assert_eq!(solve(&problem, 1), Err(Error::ScoreOverflow));
}

#[test]
fn invalid_weights_are_rejected() {
    let problem = Problem {
        variables: vec![Variable {
            name: "resource".into(),
            domain: vec![0],
        }],
        constraints: vec![Constraint::PreferValue {
            variable: 0,
            value: 1,
            soft_weight: 0,
        }],
    };

    assert_eq!(validate(&problem), Err(Error::InvalidWeight));
}

#[test]
fn offset_comparison_handles_integer_extremes() {
    let problem = Problem {
        variables: vec![
            Variable {
                name: "first".into(),
                domain: vec![i64::MAX],
            },
            Variable {
                name: "second".into(),
                domain: vec![i64::MIN],
            },
        ],
        constraints: vec![Constraint::NotEqualOffset {
            left: 0,
            right: 1,
            offset: 0,
            hard_weight: 1,
        }],
    };

    assert_eq!(
        score(&problem, &[i64::MAX, i64::MIN]),
        Ok(Score { hard: 0, soft: 0 })
    );
}

#[test]
fn empty_problem_has_one_empty_assignment() {
    let problem = Problem {
        variables: vec![],
        constraints: vec![],
    };
    let result = solve(&problem, 1).unwrap().unwrap();

    assert!(result.values.is_empty());
    assert_eq!(result.evaluated, 1);
    assert_eq!(result.score, Score { hard: 0, soft: 0 });
}

#[test]
fn zero_budget_rejects_even_an_empty_problem() {
    let problem = Problem {
        variables: vec![],
        constraints: vec![],
    };
    assert_eq!(solve(&problem, 0), Err(Error::TooManyAssignments));
}

#[test]
fn offset_penalty_overflow_is_reported() {
    let problem = Problem {
        variables: vec![
            Variable {
                name: "left".into(),
                domain: vec![0],
            },
            Variable {
                name: "right".into(),
                domain: vec![0],
            },
        ],
        constraints: vec![
            Constraint::NotEqualOffset {
                left: 0,
                right: 1,
                offset: 0,
                hard_weight: i64::MAX,
            },
            Constraint::NotEqualOffset {
                left: 0,
                right: 1,
                offset: 0,
                hard_weight: 2,
            },
        ],
    };

    assert_eq!(score(&problem, &[0, 0]), Err(Error::ScoreOverflow));
}

#[test]
fn reference_search_evaluates_every_combination() {
    let problem = Problem {
        variables: vec![
            Variable {
                name: "a".into(),
                domain: vec![0, 1, 2],
            },
            Variable {
                name: "b".into(),
                domain: vec![0, 1],
            },
            Variable {
                name: "c".into(),
                domain: vec![0, 1],
            },
        ],
        constraints: vec![],
    };
    let result = solve(&problem, 12).unwrap().unwrap();
    assert_eq!(result.evaluated, 12);
    assert_eq!(result.values, vec![0, 0, 0]);
}
