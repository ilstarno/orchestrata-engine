use orchestrata_solver::{
    CompiledProblem, Constraint, CountValueError, CountValueTier, Error, IncrementalScorer,
    Problem, Score, Solution, SolutionValidationError, Variable, score, solve, validate,
    verify_solution,
};

fn coverage(min: usize, max: usize, weight: i64, tier: CountValueTier) -> Constraint {
    Constraint::CountValue {
        variables: vec![0, 1, 2],
        value: 1,
        min,
        max,
        weight,
        tier,
    }
}

fn model(constraints: Vec<Constraint>) -> Problem {
    Problem {
        variables: (0..3)
            .map(|index| Variable {
                name: format!("worker_{index}"),
                domain: vec![0, 1],
            })
            .collect(),
        constraints,
    }
}

#[test]
fn solves_within_hard_coverage_while_optimizing_soft_coverage() {
    let input = model(vec![
        coverage(1, 2, 10, CountValueTier::Hard),
        coverage(2, 3, 3, CountValueTier::Soft),
        Constraint::PreferValue {
            variable: 0,
            value: 0,
            soft_weight: 2,
        },
    ]);
    let result = solve(&input, 8).unwrap().unwrap();
    assert_eq!(result.values, [0, 1, 1]);
    assert_eq!(result.score, Score { hard: 0, soft: 0 });
    assert_eq!(result.evaluated, 8);
    assert_eq!(solve(&input, 7), Err(Error::TooManyAssignments));
    verify_solution(&input, &result).unwrap();

    let invalid = Solution {
        values: vec![1, 1, 1],
        ..result.clone()
    };
    assert_eq!(
        verify_solution(&input, &invalid),
        Err(SolutionValidationError::HardConstraintViolation(0))
    );
    let mis_scored = Solution {
        values: vec![0, 0, 1],
        ..result
    };
    assert_eq!(
        verify_solution(&input, &mis_scored),
        Err(SolutionValidationError::ScoreMismatch)
    );
}

#[test]
fn exact_search_can_prove_valid_coverage_infeasible() {
    let mut input = model(vec![coverage(2, 3, 4, CountValueTier::Hard)]);
    input.variables[0].domain = vec![0];
    input.variables[1].domain = vec![0];
    validate(&input).unwrap();
    assert_eq!(solve(&input, 2).unwrap(), None);
}

#[test]
fn incremental_trials_match_full_scoring_across_all_assignments() {
    let input = model(vec![
        coverage(1, 2, 5, CountValueTier::Hard),
        coverage(2, 2, 4, CountValueTier::Soft),
    ]);
    let compiled = CompiledProblem::new(&input).unwrap();
    let mut scorer = IncrementalScorer::new(&compiled, &[0, 0, 0]).unwrap();
    assert_eq!(scorer.score().unwrap(), Score { hard: -5, soft: -8 });
    // Gray-code traversal flips one value at a time and visits every assignment.
    for (index, next) in [(2, 1), (1, 1), (2, 0), (0, 1), (2, 1), (1, 0), (2, 0)] {
        let previous = scorer.values().to_vec();
        let mut expected = previous.clone();
        expected[index] = next;
        {
            let trial = scorer.try_move(index, next).unwrap();
            assert_eq!(trial.score(), score(&input, &expected));
        }
        assert_eq!(scorer.values(), previous);
        scorer.try_move(index, next).unwrap().commit();
        assert_eq!(scorer.values(), expected);
        assert_eq!(scorer.score(), score(&input, &expected));
    }
}

#[test]
fn invalid_rules_include_the_rule_index_and_reason() {
    for (variables, min, max, weight, reason) in [
        (vec![], 0, 0, 1, CountValueError::EmptyScope),
        (vec![0], 1, 0, 1, CountValueError::InvalidBounds),
        (vec![0], 0, 2, 1, CountValueError::InvalidBounds),
        (vec![0], 0, 1, 0, CountValueError::InvalidWeight),
        (vec![0], 0, 1, -1, CountValueError::InvalidWeight),
        (vec![0, 3], 0, 1, 1, CountValueError::UnknownVariable(3)),
        (vec![1, 1], 0, 1, 1, CountValueError::DuplicateVariable(1)),
    ] {
        let bad = Constraint::CountValue {
            variables,
            value: 1,
            min,
            max,
            weight,
            tier: CountValueTier::Hard,
        };
        let input = model(vec![coverage(0, 1, 1, CountValueTier::Hard), bad]);
        assert_eq!(
            validate(&input),
            Err(Error::InvalidCountValue {
                constraint: 1,
                reason
            })
        );
    }
}

#[test]
fn wide_penalties_do_not_hide_representable_optima_or_wrap() {
    let valid_boundary = model(vec![
        coverage(1, 3, i64::MAX, CountValueTier::Soft),
        coverage(1, 3, 1, CountValueTier::Soft),
    ]);
    assert_eq!(score(&valid_boundary, &[0, 0, 0]).unwrap().soft, i64::MIN);
    let mut input = model(vec![coverage(3, 3, i64::MAX, CountValueTier::Soft)]);
    assert_eq!(score(&input, &[0, 0, 0]), Err(Error::ScoreOverflow));
    let compiled = CompiledProblem::new(&input).unwrap();
    assert_eq!(
        IncrementalScorer::new(&compiled, &[0, 0, 0])
            .unwrap()
            .score(),
        Err(Error::ScoreOverflow)
    );
    assert_eq!(solve(&input, 8).unwrap().unwrap().values, [1, 1, 1]);
    input.variables[0].domain = vec![0];
    input.variables[1].domain = vec![0];
    assert_eq!(solve(&input, 2), Err(Error::ScoreOverflow));
}
