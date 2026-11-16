use std::{ffi::CStr, ptr};

use orchestrata_ffi::{Status, orchestrata_free_json_v1, orchestrata_solve_json_v1};
use orchestrata_solver::{
    CompiledProblem, Constraint, Error, IncrementalScorer, PrecedenceError, Problem, Score,
    SearchOutcome, SearchStop, Solution, SolutionValidationError, Variable, score, search_locally,
    solve, validate, verify_solution,
};

fn var(name: &str, domain: &[i64]) -> Variable {
    Variable {
        name: name.into(),
        domain: domain.to_vec(),
    }
}

const fn precedes(before: usize, after: usize, min_gap: u64) -> Constraint {
    Constraint::Precedence {
        before,
        after,
        min_gap,
    }
}

fn rental_turnaround() -> Problem {
    Problem {
        variables: vec![var("return", &[100, 103]), var("pickup", &[102, 104, 106])],
        constraints: vec![
            precedes(0, 1, 3),
            Constraint::PreferValue {
                variable: 0,
                value: 103,
                soft_weight: 4,
            },
            Constraint::PreferValue {
                variable: 1,
                value: 104,
                soft_weight: 1,
            },
        ],
    }
}

#[test]
fn rental_turnaround_enforces_the_minimum_start_gap() {
    let problem = rental_turnaround();
    assert_eq!(
        score(&problem, &[103, 104]),
        Ok(Score { hard: -2, soft: 0 })
    );
    assert_eq!(
        score(&problem, &[103, 106]),
        Ok(Score { hard: 0, soft: -1 })
    );
    assert_eq!(solve(&problem, 5), Err(Error::TooManyAssignments));

    let actual = solve(&problem, 6).unwrap().unwrap();
    assert_eq!(actual.values, [103, 106]);
    assert_eq!(actual.evaluated, 6);
    verify_solution(&problem, &actual).unwrap();

    let false_plan = Solution {
        values: vec![103, 104],
        ..actual
    };
    assert_eq!(
        verify_solution(&problem, &false_plan),
        Err(SolutionValidationError::HardConstraintViolation(0))
    );
}

#[test]
fn zero_gap_allows_equality_but_positive_cycles_are_infeasible() {
    let mut problem = Problem {
        variables: vec![var("a", &[0, 1]), var("b", &[0, 1])],
        constraints: vec![precedes(0, 1, 0), precedes(1, 0, 0)],
    };
    assert_eq!(solve(&problem, 4).unwrap().unwrap().values, [0, 0]);
    problem.constraints[1] = precedes(1, 0, 1);
    assert_eq!(validate(&problem), Ok(()));
    assert_eq!(solve(&problem, 4), Ok(None));
    assert_eq!(
        search_locally(&problem, 1),
        Ok(SearchOutcome::NoFeasibleIncumbent {
            evaluated: 1,
            stop: SearchStop::EvaluationLimit
        }),
    );
}

#[test]
fn validates_distinct_in_scope_positions_before_search() {
    let mut problem = rental_turnaround();
    for (rule, reason) in [
        (precedes(2, 1, 0), PrecedenceError::UnknownBeforeVariable(2)),
        (precedes(0, 2, 0), PrecedenceError::UnknownAfterVariable(2)),
        (precedes(1, 1, 0), PrecedenceError::SameVariable(1)),
        (precedes(0, 0, 1), PrecedenceError::SameVariable(0)),
    ] {
        problem.constraints.push(rule);
        let actual = solve(&problem, 0);
        let expected = Error::InvalidPrecedence {
            constraint: 3,
            reason,
        };
        assert_eq!(actual, Err(expected.clone()));
        assert_eq!(CompiledProblem::new(&problem).err(), Some(expected.clone()));
        let supplied = Solution {
            values: vec![103, 106],
            score: Score { hard: 0, soft: -1 },
            evaluated: 1,
        };
        assert_eq!(
            verify_solution(&problem, &supplied),
            Err(SolutionValidationError::InvalidProblem(expected))
        );
        problem.constraints.pop();
    }
}

#[test]
fn signed_endpoints_and_unsigned_gap_use_widened_arithmetic() {
    let entire_range = Problem {
        variables: vec![var("first", &[i64::MIN]), var("last", &[i64::MAX])],
        constraints: vec![precedes(0, 1, u64::MAX)],
    };
    assert_eq!(
        solve(&entire_range, 1).unwrap().unwrap().values,
        [i64::MIN, i64::MAX]
    );

    for (before, after, gap, expected) in [
        (
            0,
            i64::MIN,
            0,
            Ok(Score {
                hard: i64::MIN,
                soft: 0,
            }),
        ),
        (0, i64::MIN, 1, Err(Error::ScoreOverflow)),
        (i64::MAX, i64::MIN, u64::MAX, Err(Error::ScoreOverflow)),
        (
            i64::MIN,
            i64::MAX - 1,
            u64::MAX,
            Ok(Score { hard: -1, soft: 0 }),
        ),
    ] {
        let problem = Problem {
            variables: vec![var("before", &[before]), var("after", &[after])],
            constraints: vec![precedes(0, 1, gap)],
        };
        let compiled = CompiledProblem::new(&problem).unwrap();
        let scorer = IncrementalScorer::new(&compiled, &[before, after]).unwrap();
        assert_eq!(score(&problem, &[before, after]), expected);
        assert_eq!(scorer.score(), expected);
        assert!(!scorer.is_feasible());
        assert_eq!(solve(&problem, 1), Ok(None));
    }
}

#[test]
fn incremental_trials_match_full_score_and_leave_only_verified_incumbents() {
    let problem = rental_turnaround();
    let compiled = CompiledProblem::new(&problem).unwrap();
    assert_eq!(compiled.incident_constraints(0), Some(&[0, 1][..]));
    assert_eq!(compiled.incident_constraints(1), Some(&[0, 2][..]));
    let mut scorer = IncrementalScorer::new(&compiled, &[100, 102]).unwrap();

    for (variable, value, commit) in [
        (0, 103, false),
        (1, 106, true),
        (0, 103, true),
        (1, 102, false),
    ] {
        let previous = scorer.values().to_vec();
        let trial = scorer.try_move(variable, value).unwrap();
        let full = score(&problem, trial.values()).unwrap();
        assert_eq!(trial.score(), Ok(full));
        assert_eq!(trial.is_feasible(), full.hard == 0);
        if commit {
            trial.commit();
            assert_eq!(scorer.values()[variable], value);
        } else {
            drop(trial);
            assert_eq!(scorer.values(), previous);
        }
        assert_eq!(scorer.score(), score(&problem, scorer.values()));
    }

    let SearchOutcome::FeasibleIncumbent { solution, .. } = search_locally(&problem, 12).unwrap()
    else {
        panic!("expected a feasible local incumbent");
    };
    verify_solution(&problem, &solution).unwrap();
}

#[test]
fn raw_abi_v1_handles_additive_precedence_and_invalid_gap() {
    let input = br#"{"variables":[{"name":"transfer_in","domain":[-9223372036854775808]},{"name":"transfer_out","domain":[9223372036854775807]}],"constraints":[{"kind":"precedence","before":0,"after":1,"min_gap":18446744073709551615}]}"#;
    let (status, body) = native_run(input);
    assert_eq!(status, Status::Ok as i32);
    let solution: serde_json::Value = serde_json::from_slice(&body.unwrap()).unwrap();
    assert_eq!(
        solution["solution"]["values"],
        serde_json::json!([i64::MIN, i64::MAX])
    );

    for wire in [
        br#"{"variables":[{"name":"a","domain":[1]}],"constraints":[{"kind":"precedence","before":0,"after":0,"min_gap":0}]}"#.as_slice(),
        br#"{"variables":[{"name":"a","domain":[1]},{"name":"b","domain":[2]}],"constraints":[{"kind":"precedence","before":0,"after":1,"min_gap":-1}]}"#.as_slice(),
    ] {
        assert_eq!(native_run(wire), (Status::InvalidProblem as i32, None));
    }
}

fn native_run(input: &[u8]) -> (i32, Option<Vec<u8>>) {
    let mut result = ptr::null_mut();
    // SAFETY: both buffers are live, aligned, and disjoint during the call.
    let status = unsafe { orchestrata_solve_json_v1(input.as_ptr(), input.len(), 1, &mut result) };
    if result.is_null() {
        return (status, None);
    }
    // SAFETY: a non-null result is native-owned NUL-terminated data released exactly once.
    let document = unsafe {
        let bytes = CStr::from_ptr(result).to_bytes().to_vec();
        orchestrata_free_json_v1(result);
        bytes
    };
    (status, Some(document))
}
