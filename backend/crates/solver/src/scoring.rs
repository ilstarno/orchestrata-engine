use crate::{Constraint, Error, Problem, Score, validate};

pub fn score(problem: &Problem, values: &[i64]) -> Result<Score, Error> {
    validate(problem)?;

    if values.len() != problem.variables.len() {
        return Err(Error::InvalidVariable(values.len()));
    }

    for (index, (variable, value)) in problem.variables.iter().zip(values).enumerate() {
        if !variable.domain.contains(value) {
            return Err(Error::InvalidVariable(index));
        }
    }

    let mut result = Score { hard: 0, soft: 0 };

    for constraint in &problem.constraints {
        match constraint {
            Constraint::AllDifferent {
                variables,
                hard_weight,
            } => {
                for i in 0..variables.len() {
                    for j in i + 1..variables.len() {
                        if values[variables[i]] == values[variables[j]] {
                            result.hard = result
                                .hard
                                .checked_sub(*hard_weight)
                                .ok_or(Error::ScoreOverflow)?;
                        }
                    }
                }
            }
            Constraint::NotEqualOffset {
                left,
                right,
                offset,
                hard_weight,
            } => {
                if i128::from(values[*left]) - i128::from(values[*right]) == i128::from(*offset) {
                    result.hard = result
                        .hard
                        .checked_sub(*hard_weight)
                        .ok_or(Error::ScoreOverflow)?;
                }
            }
            Constraint::PreferValue {
                variable,
                value,
                soft_weight,
            } => {
                if values[*variable] != *value {
                    result.soft = result
                        .soft
                        .checked_sub(*soft_weight)
                        .ok_or(Error::ScoreOverflow)?;
                }
            }
        }
    }

    Ok(result)
}
