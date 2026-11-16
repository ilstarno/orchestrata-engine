use orchestrata_solver::{
    AssignmentChange, AssignmentChangeReason, Constraint, Problem, Score, Solution,
    SolutionComparisonError, SolutionValidationError, Variable, compare_solutions,
};

fn crew_model() -> Problem {
    Problem {
        variables: vec![
            Variable {
                name: "inspection".into(),
                domain: vec![11, 22],
            },
            Variable {
                name: "repair".into(),
                domain: vec![22, 33],
            },
            Variable {
                name: "delivery".into(),
                domain: vec![44],
            },
        ],
        constraints: vec![
            Constraint::AllDifferent {
                variables: vec![0, 1, 2],
                hard_weight: 1,
            },
            Constraint::PreferValue {
                variable: 1,
                value: 22,
                soft_weight: 7,
            },
        ],
    }
}

fn plan(values: &[i64], soft: i64) -> Solution {
    Solution {
        values: values.to_vec(),
        score: Score { hard: 0, soft },
        evaluated: 1,
    }
}

#[test]
fn reports_only_changes_in_order_without_inventing_a_cause() {
    let previous = crew_model();
    let mut updated = previous.clone();
    updated.variables[0].domain = vec![22];

    let result = compare_solutions(
        &previous,
        &plan(&[11, 22, 44], 0),
        &updated,
        &plan(&[22, 33, 44], -7),
    )
    .unwrap();

    assert_eq!(result.previous_score, Score { hard: 0, soft: 0 });
    assert_eq!(result.updated_score, Score { hard: 0, soft: -7 });
    assert_eq!(
        result.changes,
        [
            AssignmentChange {
                variable_index: 0,
                previous_value: 11,
                updated_value: 22,
                reason: AssignmentChangeReason::PreviousValueOutsideUpdatedDomain,
            },
            AssignmentChange {
                variable_index: 1,
                previous_value: 22,
                updated_value: 33,
                reason: AssignmentChangeReason::ReasonNotEstablished,
            },
        ]
    );
}

#[test]
fn unchanged_assignments_can_have_different_verified_scores() {
    let previous = crew_model();
    let mut updated = previous.clone();
    updated.constraints.remove(1);
    let values = [11, 33, 44];

    let result =
        compare_solutions(&previous, &plan(&values, -7), &updated, &plan(&values, 0)).unwrap();

    assert!(result.changes.is_empty());
    assert_eq!(result.previous_score.soft, -7);
    assert_eq!(result.updated_score.soft, 0);
}

#[test]
fn rejects_unrelated_models_even_when_both_plans_are_feasible() {
    let previous = crew_model();
    let mut updated = previous.clone();
    let valid = plan(&[11, 22, 44], 0);
    updated.variables[0].name = "inventory".into();

    assert_eq!(
        compare_solutions(&previous, &valid, &updated, &valid),
        Err(SolutionComparisonError::VariableIdentityMismatch(0))
    );
}

#[test]
fn rejects_reordered_or_resized_model_variables() {
    let previous = crew_model();
    let mut updated = previous.clone();
    let valid = plan(&[11, 22, 44], 0);
    updated.variables.swap(0, 1);
    updated.constraints.clear();

    assert_eq!(
        compare_solutions(&previous, &valid, &updated, &plan(&[22, 11, 44], 0)),
        Err(SolutionComparisonError::VariableIdentityMismatch(0))
    );

    updated.variables.pop();
    assert_eq!(
        compare_solutions(&previous, &valid, &updated, &plan(&[22, 11], 0)),
        Err(SolutionComparisonError::VariableCountMismatch)
    );
}

#[test]
fn rejects_ambiguous_variable_identity_without_exposing_its_name() {
    let mut problem = crew_model();
    let valid = plan(&[11, 22, 44], 0);

    problem.variables[1].name = " inspection  ".into();
    assert_eq!(
        compare_solutions(&problem, &valid, &problem, &valid),
        Err(SolutionComparisonError::InvalidVariableIdentity(1))
    );

    problem.variables[0].name = " ".into();
    assert_eq!(
        compare_solutions(&problem, &valid, &problem, &valid),
        Err(SolutionComparisonError::InvalidVariableIdentity(0))
    );
}

#[test]
fn rejects_a_forged_previous_score_or_updated_hard_violation() {
    let problem = crew_model();
    let valid = plan(&[11, 22, 44], 0);

    assert_eq!(
        compare_solutions(&problem, &plan(&[11, 22, 44], -7), &problem, &valid),
        Err(SolutionComparisonError::InvalidPreviousSolution(
            SolutionValidationError::ScoreMismatch
        ))
    );

    assert_eq!(
        compare_solutions(&problem, &valid, &problem, &plan(&[22, 22, 44], 0)),
        Err(SolutionComparisonError::InvalidUpdatedSolution(
            SolutionValidationError::HardConstraintViolation(0)
        ))
    );
}
