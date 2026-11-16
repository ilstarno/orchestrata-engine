use orchestrata_solver::{
    Constraint, Error, Problem, Score, SearchOutcome, SearchStop, Variable, search_locally, solve,
    verify_solution,
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
