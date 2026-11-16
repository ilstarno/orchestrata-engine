use std::collections::HashSet;

use crate::{Constraint, Problem, Score, Solution, SolutionValidationError, validate};

/// Checks feasibility and recomputes the score without using the search scorer.
///
/// This validates the assignment and claimed score, not optimality or search effort.
pub fn verify_solution(
    problem: &Problem,
    solution: &Solution,
) -> Result<(), SolutionValidationError> {
    validate(problem).map_err(SolutionValidationError::InvalidProblem)?;
    let values = &solution.values;
    if values.len() != problem.variables.len() {
        return Err(SolutionValidationError::WrongValueCount);
    }

    for (index, (variable, value)) in problem.variables.iter().zip(values).enumerate() {
        if !variable.domain.contains(value) {
            return Err(SolutionValidationError::ValueOutsideDomain(index));
        }
    }

    let mut soft_penalty = 0_i128;
    for (index, constraint) in problem.constraints.iter().enumerate() {
        match constraint {
            Constraint::AllDifferent { variables, .. } => {
                let mut unique = HashSet::with_capacity(variables.len());
                if !variables
                    .iter()
                    .all(|&variable| unique.insert(values[variable]))
                {
                    return Err(SolutionValidationError::HardConstraintViolation(index));
                }
            }
            Constraint::NotEqualOffset {
                left,
                right,
                offset,
                ..
            } => {
                if i128::from(values[*left]) == i128::from(values[*right]) + i128::from(*offset) {
                    return Err(SolutionValidationError::HardConstraintViolation(index));
                }
            }
            Constraint::PreferValue {
                variable,
                value,
                soft_weight,
            } => {
                if values[*variable] != *value {
                    soft_penalty += i128::from(*soft_weight);
                }
            }
        }
    }

    let soft = i64::try_from(-soft_penalty).map_err(|_| SolutionValidationError::ScoreOverflow)?;
    if solution.score != (Score { hard: 0, soft }) {
        return Err(SolutionValidationError::ScoreMismatch);
    }

    Ok(())
}
