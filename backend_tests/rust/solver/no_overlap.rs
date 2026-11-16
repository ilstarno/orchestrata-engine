use orchestrata_solver::{
    CompiledProblem, Constraint, Error, IncrementalScorer, NoOverlapError, Problem,
    ScheduledInterval, Score, Solution, SolutionValidationError, Variable, score, solve, validate,
    verify_solution,
};

fn interval(start: usize, duration: u64) -> ScheduledInterval {
    ScheduledInterval { start, duration }
}

fn room(starts: &[&[i64]], durations: &[u64]) -> Problem {
    Problem {
        variables: starts
            .iter()
            .enumerate()
            .map(|(index, domain)| Variable {
                name: format!("booking_{index}"),
                domain: domain.to_vec(),
            })
            .collect(),
        constraints: vec![Constraint::NoOverlap {
            intervals: durations
                .iter()
                .enumerate()
                .map(|(i, &d)| interval(i, d))
                .collect(),
        }],
    }
}

#[test]
fn equipment_bookings_can_touch_but_may_not_overlap() {
    let model = room(&[&[0, 30], &[20, 30]], &[30, 15]);
    let result = solve(&model, 4).unwrap().unwrap();
    assert_eq!(result.values, [0, 30]);
    assert_eq!(result.score, Score { hard: 0, soft: 0 });
    assert_eq!(result.evaluated, 4);
    verify_solution(&model, &result).unwrap();
    let forged = Solution {
        values: vec![0, 20],
        ..result
    };
    assert_eq!(
        verify_solution(&model, &forged),
        Err(SolutionValidationError::HardConstraintViolation(0))
    );
    assert_eq!(score(&model, &forged.values).unwrap().hard, -10);
    assert_eq!(solve(&model, 3), Err(Error::TooManyAssignments));
    assert_eq!(solve(&room(&[&[0], &[20]], &[30, 15]), 1), Ok(None));
}

#[test]
fn nested_triple_overlap_sums_each_pair_and_incremental_moves_match() {
    let model = room(&[&[0, 10], &[2, 15], &[4, 20]], &[10, 5, 2]);
    let compiled = CompiledProblem::new(&model).unwrap();
    let mut scorer = IncrementalScorer::new(&compiled, &[0, 2, 4]).unwrap();
    assert_eq!(scorer.score().unwrap().hard, -9); // 5 + 2 + 2
    // Gray-code traversal visits all eight start configurations.
    for (index, value) in [(2, 20), (1, 15), (2, 4), (0, 10), (2, 20), (1, 2), (2, 4)] {
        let previous = scorer.values().to_vec();
        let mut next = previous.clone();
        next[index] = value;
        {
            let trial = scorer.try_move(index, value).unwrap();
            assert_eq!(trial.score(), score(&model, &next));
        }
        assert_eq!(scorer.values(), previous);
        scorer.try_move(index, value).unwrap().commit();
        assert_eq!(scorer.score(), score(&model, &next));
    }
}

#[test]
fn widened_endpoints_and_empty_intervals_keep_boundary_meanings() {
    let model = room(&[&[i64::MIN], &[i64::MAX], &[i64::MAX]], &[u64::MAX, 1, 0]);
    assert_eq!(solve(&model, 1).unwrap().unwrap().score.hard, 0);
    let model = room(&[&[i64::MAX], &[i64::MAX]], &[u64::MAX, 1]);
    assert_eq!(score(&model, &[i64::MAX; 2]).unwrap().hard, -1);
    let model = room(&[&[i64::MIN], &[i64::MIN]], &[u64::MAX; 2]);
    assert_eq!(solve(&model, 1), Ok(None));
    assert_eq!(score(&model, &[i64::MIN; 2]), Err(Error::ScoreOverflow));
    let compiled = CompiledProblem::new(&model).unwrap();
    assert_eq!(
        IncrementalScorer::new(&compiled, &[i64::MIN; 2])
            .unwrap()
            .score(),
        Err(Error::ScoreOverflow)
    );
}

#[test]
fn rejects_short_duplicate_and_unknown_scopes_with_indexed_reasons() {
    for (intervals, reason) in [
        (vec![interval(0, 1)], NoOverlapError::TooFewIntervals),
        (
            vec![interval(1, 0), interval(1, 1)],
            NoOverlapError::DuplicateStart(1),
        ),
        (
            vec![interval(0, 1), interval(2, 1)],
            NoOverlapError::UnknownStart(2),
        ),
    ] {
        let mut model = room(&[&[0], &[1]], &[1, 1]);
        model.constraints.push(Constraint::NoOverlap { intervals });
        assert_eq!(
            validate(&model),
            Err(Error::InvalidNoOverlap {
                constraint: 1,
                reason
            })
        );
    }
}
