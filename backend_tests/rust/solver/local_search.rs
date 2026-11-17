use orchestrata_solver::{
    Constraint, Error, Problem, Score, SearchOutcome, SearchStop, Solution, Variable, score,
    search_locally, search_locally_with_swaps, solve, verify_solution,
};

fn variable(name: &str, domain: &[i64]) -> Variable {
    Variable {
        name: name.to_owned(),
        domain: domain.to_vec(),
    }
}

fn all_different(weight: i64) -> Constraint {
    Constraint::AllDifferent {
        variables: vec![0, 1],
        hard_weight: weight,
    }
}

#[test]
fn bounded_search_counts_initial_and_trials_and_preserves_feasible_incumbent() {
    let problem = Problem {
        variables: vec![variable("first", &[0, 1]), variable("second", &[0, 1])],
        constraints: vec![all_different(1)],
    };

    for budget in [0, 1] {
        assert_eq!(
            search_locally(&problem, budget),
            Ok(SearchOutcome::NoFeasibleIncumbent {
                evaluated: budget,
                stop: SearchStop::EvaluationLimit,
            })
        );
    }

    for budget in 2..=6 {
        let SearchOutcome::FeasibleIncumbent { solution, stop } =
            search_locally(&problem, budget).unwrap()
        else {
            panic!("the first nontrivial move makes the assignment feasible");
        };
        assert_eq!(solution.values, [1, 0]);
        assert_eq!(solution.evaluated, 4.min(budget));
        assert_eq!(
            stop,
            if budget < 4 {
                SearchStop::EvaluationLimit
            } else {
                SearchStop::NeighborhoodExhausted
            }
        );
        assert_eq!(verify_solution(&problem, &solution), Ok(()));
    }
}

#[test]
fn exhausted_neighborhood_without_incumbent_does_not_claim_infeasibility() {
    let problem = Problem {
        variables: vec![variable("first", &[0, 1]), variable("second", &[0, 2])],
        constraints: vec![
            all_different(1),
            Constraint::NotEqualOffset {
                left: 0,
                right: 1,
                offset: 1,
                hard_weight: 2,
            },
            Constraint::NotEqualOffset {
                left: 0,
                right: 1,
                offset: -2,
                hard_weight: 2,
            },
        ],
    };

    assert_eq!(
        search_locally(&problem, 100),
        Ok(SearchOutcome::NoFeasibleIncumbent {
            evaluated: 3,
            stop: SearchStop::NeighborhoodExhausted,
        })
    );
    assert_eq!(solve(&problem, 4).unwrap().unwrap().values, [1, 2]);
}

#[test]
fn exhausted_feasible_neighborhood_does_not_claim_global_optimality() {
    let problem = Problem {
        variables: vec![variable("first", &[0, 1]), variable("second", &[1, 0])],
        constraints: vec![
            all_different(1),
            Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight: i64::MAX,
            },
        ],
    };

    let result = search_locally(&problem, 16).unwrap();
    assert_eq!(result, search_locally(&problem, 16).unwrap());

    let SearchOutcome::FeasibleIncumbent { solution, stop } = result else {
        panic!("the constructed assignment is feasible");
    };
    assert_eq!(stop, SearchStop::NeighborhoodExhausted);
    assert_eq!(solution.values, [0, 1]);
    assert_eq!(
        solution.score,
        Score {
            hard: 0,
            soft: -i64::MAX
        }
    );
    assert_eq!(verify_solution(&problem, &solution), Ok(()));
    assert_eq!(solve(&problem, 4).unwrap().unwrap().score.soft, 0);
}

#[test]
fn wide_ranking_can_improve_from_an_unrepresentable_feasible_score() {
    let problem = Problem {
        variables: vec![variable("first", &[0, 1])],
        constraints: [i64::MAX, 2]
            .map(|soft_weight| Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight,
            })
            .into(),
    };

    assert_eq!(search_locally(&problem, 1), Err(Error::ScoreOverflow));
    let SearchOutcome::FeasibleIncumbent { solution, stop } = search_locally(&problem, 2).unwrap()
    else {
        panic!("wide comparison should find the representable improvement");
    };
    assert_eq!(stop, SearchStop::EvaluationLimit);
    assert_eq!(solution.values, [1]);
    assert_eq!(solution.score.soft, 0);
    assert_eq!(verify_solution(&problem, &solution), Ok(()));
}

#[test]
fn empty_problem_has_one_candidate_and_invalid_problem_is_rejected_even_at_zero_budget() {
    let problem = Problem {
        variables: vec![],
        constraints: vec![],
    };
    let SearchOutcome::FeasibleIncumbent { solution, stop } = search_locally(&problem, 1).unwrap()
    else {
        panic!("an empty assignment is feasible");
    };
    assert_eq!(stop, SearchStop::NeighborhoodExhausted);
    assert_eq!(solution.evaluated, 1);
    assert_eq!(verify_solution(&problem, &solution), Ok(()));

    let invalid = Problem {
        variables: vec![variable("empty", &[])],
        constraints: vec![],
    };
    assert_eq!(search_locally(&invalid, 0), Err(Error::EmptyDomain(0)));
}

#[test]
fn opt_in_swap_escapes_feasible_tie_barrier_without_changing_default() {
    use orchestrata_solver::{ResourceLimit, ResourceLoad};
    let problem = Problem {
        variables: vec![
            variable("first", &[i64::MIN, i64::MAX]),
            variable("second", &[i64::MAX, i64::MIN]),
        ],
        constraints: vec![
            all_different(i64::MAX),
            Constraint::ResourceCapacity {
                items: vec![
                    ResourceLoad {
                        variable: 0,
                        load: u64::MAX / 2,
                    },
                    ResourceLoad {
                        variable: 1,
                        load: u64::MAX / 2,
                    },
                ],
                limits: vec![
                    ResourceLimit {
                        resource: i64::MIN,
                        capacity: u64::MAX / 2,
                    },
                    ResourceLimit {
                        resource: i64::MAX,
                        capacity: u64::MAX / 2,
                    },
                ],
            },
            Constraint::PreferValue {
                variable: 0,
                value: i64::MAX,
                soft_weight: i64::MAX,
            },
        ],
    };
    let initial = Solution {
        values: vec![i64::MIN, i64::MAX],
        score: Score {
            hard: 0,
            soft: -i64::MAX,
        },
        evaluated: 3,
    };
    assert_eq!(
        search_locally(&problem, 7),
        Ok(SearchOutcome::FeasibleIncumbent {
            solution: initial.clone(),
            stop: SearchStop::NeighborhoodExhausted,
        })
    );
    assert_eq!(
        search_locally_with_swaps(&problem, 3),
        Ok(SearchOutcome::FeasibleIncumbent {
            solution: initial,
            stop: SearchStop::EvaluationLimit,
        })
    );
    for budget in 4..=8 {
        let expected = solve(&problem, 4).unwrap().unwrap();
        assert_eq!(
            search_locally_with_swaps(&problem, budget),
            Ok(SearchOutcome::FeasibleIncumbent {
                solution: Solution {
                    evaluated: budget.min(7),
                    ..expected
                },
                stop: if budget < 7 {
                    SearchStop::EvaluationLimit
                } else {
                    SearchStop::NeighborhoodExhausted
                },
            })
        );
    }
}

fn exhaustive_neighborhood(problem: &Problem, budget: u64) -> SearchOutcome {
    if budget == 0 {
        return SearchOutcome::NoFeasibleIncumbent {
            evaluated: 0,
            stop: SearchStop::EvaluationLimit,
        };
    }
    let mut values: Vec<_> = problem.variables.iter().map(|v| v.domain[0]).collect();
    let rank = |values: &[i64]| {
        let s = score(problem, values).unwrap();
        (s.hard, s.soft)
    };
    let mut evaluated = 1;
    let stop = 'passes: loop {
        let mut candidates = vec![];
        for (i, variable) in problem.variables.iter().enumerate() {
            for &value in &variable.domain {
                if value != values[i] {
                    let mut next = values.clone();
                    next[i] = value;
                    candidates.push(next);
                }
            }
        }
        for i in 0..values.len() {
            for j in i + 1..values.len() {
                let mut next = values.clone();
                next.swap(i, j);
                if next != values
                    && problem.variables[i].domain.contains(&next[i])
                    && problem.variables[j].domain.contains(&next[j])
                {
                    candidates.push(next);
                }
            }
        }
        for candidate in candidates {
            if evaluated == budget {
                break 'passes SearchStop::EvaluationLimit;
            }
            evaluated += 1;
            if rank(&candidate) > rank(&values) {
                values = candidate;
                continue 'passes;
            }
        }
        break SearchStop::NeighborhoodExhausted;
    };
    let score = score(problem, &values).unwrap();
    if score.hard == 0 {
        SearchOutcome::FeasibleIncumbent {
            solution: Solution {
                values,
                score,
                evaluated,
            },
            stop,
        }
    } else {
        SearchOutcome::NoFeasibleIncumbent { evaluated, stop }
    }
}

#[test]
fn swap_deltas_match_full_scores_across_deterministic_bounded_models() {
    use orchestrata_solver::{CountValueTier, ResourceLimit, ResourceLoad, ScheduledInterval};
    let mut state = 0x628c_c9b9_0ca1_a928_u64;
    let mut random = |n: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % n
    };
    for case in 0..256 {
        let vars: Vec<_> = (0..4)
            .map(|i| {
                let rotated = random(3) as i64;
                let mut domain = vec![rotated, (rotated + 1) % 3, (rotated + 2) % 3];
                domain.truncate(1 + random(3) as usize);
                variable(&i.to_string(), &domain)
            })
            .collect();
        let declared: Vec<_> = (0..3)
            .filter(|id| vars[1].domain.contains(id) || vars[2].domain.contains(id))
            .collect();
        let problem = Problem {
            variables: vars,
            constraints: vec![
                all_different(3),
                Constraint::NotEqualOffset {
                    left: 1,
                    right: 2,
                    offset: random(5) as i64 - 2,
                    hard_weight: 2,
                },
                Constraint::CountValue {
                    variables: vec![0, 1, 2, 3],
                    value: 1,
                    min: 1,
                    max: 2,
                    weight: 3,
                    tier: CountValueTier::Soft,
                },
                Constraint::PreferValue {
                    variable: random(4) as usize,
                    value: random(3) as i64,
                    soft_weight: 4,
                },
                Constraint::Precedence {
                    before: 2,
                    after: 3,
                    min_gap: random(2),
                },
                Constraint::NoOverlap {
                    intervals: vec![
                        ScheduledInterval {
                            start: 0,
                            duration: random(3),
                        },
                        ScheduledInterval {
                            start: 3,
                            duration: random(3),
                        },
                    ],
                },
                Constraint::ResourceCapacity {
                    items: vec![
                        ResourceLoad {
                            variable: 1,
                            load: 1,
                        },
                        ResourceLoad {
                            variable: 2,
                            load: 2,
                        },
                    ],
                    limits: declared
                        .into_iter()
                        .map(|resource| ResourceLimit {
                            resource,
                            capacity: 1 + random(3),
                        })
                        .collect(),
                },
            ],
        };
        for budget in [0, 1, 2, 3, 4, 5, 7, 15, 100] {
            assert_eq!(
                search_locally_with_swaps(&problem, budget).unwrap(),
                exhaustive_neighborhood(&problem, budget),
                "case {case}, budget {budget}"
            );
        }
    }
}
