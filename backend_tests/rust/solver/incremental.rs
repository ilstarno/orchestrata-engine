use std::panic::{AssertUnwindSafe, catch_unwind};

use orchestrata_solver::{
    CompiledProblem, Constraint, Error, IncrementalScorer, Problem, Score, Variable, score,
};

fn fixture() -> Problem {
    Problem {
        variables: (0..4)
            .map(|index| Variable {
                name: format!("variable_{index}"),
                domain: vec![i64::MIN, -1, 0, 1, i64::MAX],
            })
            .collect(),
        constraints: vec![
            Constraint::AllDifferent {
                variables: vec![0, 1, 0, 2],
                hard_weight: 3,
            },
            Constraint::NotEqualOffset {
                left: 0,
                right: 1,
                offset: i64::MAX,
                hard_weight: 9,
            },
            Constraint::NotEqualOffset {
                left: 2,
                right: 2,
                offset: 0,
                hard_weight: 4,
            },
            Constraint::PreferValue {
                variable: 1,
                value: -1,
                soft_weight: 5,
            },
            Constraint::PreferValue {
                variable: 2,
                value: i64::MAX,
                soft_weight: 12,
            },
        ],
    }
}

#[test]
fn compiler_indexes_repeated_references_once_in_source_order() {
    let problem = fixture();
    let compiled = CompiledProblem::new(&problem).unwrap();

    assert_eq!(compiled.incident_constraints(0), Some([0, 1].as_slice()));
    assert_eq!(compiled.incident_constraints(1), Some([0, 1, 3].as_slice()));
    assert_eq!(compiled.incident_constraints(2), Some([0, 2, 4].as_slice()));
    assert_eq!(compiled.incident_constraints(3), Some([].as_slice()));
    assert_eq!(compiled.incident_constraints(4), None);
}

#[test]
fn deterministic_move_sequence_matches_full_score_for_trials_and_undo() {
    let problem = fixture();
    let compiled = CompiledProblem::new(&problem).unwrap();
    let mut values = vec![0, 1, -1, i64::MAX];
    let mut scorer = IncrementalScorer::new(&compiled, &values).unwrap();

    let mut seed = 17_u64;
    for step in 0..500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let variable = ((seed >> 32) % 4) as usize;
        let domain = &problem.variables[variable].domain;
        let next = domain[((seed >> 48) % domain.len() as u64) as usize];

        let previous = values[variable];
        values[variable] = next;
        let trial = scorer.try_move(variable, next).unwrap();
        assert_eq!(trial.values(), values);
        let expected = score(&problem, &values);
        assert_eq!(trial.score(), expected, "candidate at move {step}");
        assert_eq!(trial.is_feasible(), expected.unwrap().hard == 0);

        if step % 3 == 0 {
            trial.commit();
        } else {
            drop(trial);
            values[variable] = previous;
        }

        assert_eq!(scorer.values(), values);
        assert_eq!(
            scorer.score(),
            score(&problem, &values),
            "state after move {step}"
        );
    }
}

#[test]
fn rejects_invalid_inputs_before_mutation_and_undoes_during_unwind() {
    let mut invalid_problem = fixture();
    invalid_problem.constraints.push(Constraint::PreferValue {
        variable: 4,
        value: 0,
        soft_weight: 1,
    });
    assert_eq!(
        CompiledProblem::new(&invalid_problem).err(),
        Some(Error::InvalidVariable(4))
    );

    let problem = fixture();
    let compiled = CompiledProblem::new(&problem).unwrap();
    assert_eq!(
        IncrementalScorer::new(&compiled, &[0]).err(),
        Some(Error::InvalidVariable(1))
    );
    assert_eq!(
        IncrementalScorer::new(&compiled, &[0, 2, 0, 0]).err(),
        Some(Error::InvalidVariable(1))
    );

    let mut scorer = IncrementalScorer::new(&compiled, &[0, 0, 0, 0]).unwrap();
    let original = scorer.score();
    assert_eq!(scorer.try_move(4, 0).err(), Some(Error::InvalidVariable(4)));
    assert_eq!(scorer.try_move(0, 2).err(), Some(Error::InvalidVariable(0)));

    let rejected = catch_unwind(AssertUnwindSafe(|| {
        let trial = scorer.try_move(0, i64::MIN).unwrap();
        assert_ne!(trial.score(), original);
        panic!("cancel simulated caller operation");
    }));
    assert_eq!(
        rejected.unwrap_err().downcast_ref::<&str>(),
        Some(&"cancel simulated caller operation")
    );
    assert_eq!(scorer.values(), [0, 0, 0, 0]);
    assert_eq!(scorer.score(), original);

    // A fresh edit uses the restored cached contributions, not just the restored total.
    scorer.try_move(1, i64::MAX).unwrap().commit();
    assert_eq!(scorer.score(), score(&problem, scorer.values()));
}

#[test]
fn wide_diagnostic_penalties_recover_without_loss() {
    let problem = Problem {
        variables: vec![Variable {
            name: "x".into(),
            domain: vec![0, 1, 2],
        }],
        constraints: [(1, i64::MAX), (1, 2), (2, 1)]
            .into_iter()
            .map(|(value, soft_weight)| Constraint::PreferValue {
                variable: 0,
                value,
                soft_weight,
            })
            .collect(),
    };
    let compiled = CompiledProblem::new(&problem).unwrap();
    let mut scorer = IncrementalScorer::new(&compiled, &[0]).unwrap();
    assert_eq!(scorer.score(), Err(Error::ScoreOverflow));
    assert!(scorer.is_feasible());

    {
        let trial = scorer.try_move(0, 1).unwrap();
        assert_eq!(trial.score(), Ok(Score { hard: 0, soft: -1 }));
    }

    assert_eq!(scorer.score(), Err(Error::ScoreOverflow));
    scorer.try_move(0, 1).unwrap().commit();
    assert_eq!(scorer.score(), score(&problem, &[1]));
}

#[test]
fn feasibility_is_not_the_same_as_score_representability() {
    let problem = Problem {
        variables: vec![
            Variable {
                name: "first".into(),
                domain: vec![0, 1],
            },
            Variable {
                name: "second".into(),
                domain: vec![0, 1],
            },
        ],
        constraints: [i64::MAX, 2]
            .into_iter()
            .map(|hard_weight| Constraint::AllDifferent {
                variables: vec![0, 1],
                hard_weight,
            })
            .collect(),
    };
    let compiled = CompiledProblem::new(&problem).unwrap();
    let mut scorer = IncrementalScorer::new(&compiled, &[0, 0]).unwrap();
    assert!(!scorer.is_feasible());
    assert_eq!(scorer.score(), Err(Error::ScoreOverflow));

    let trial = scorer.try_move(1, 1).unwrap();
    assert!(trial.is_feasible());
    assert_eq!(trial.score(), Ok(Score { hard: 0, soft: 0 }));
    trial.commit();
    assert!(scorer.is_feasible());
}
