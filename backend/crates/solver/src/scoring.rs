use super::{
    count_value_distance::distance, precedence_shortfall::shortfall,
    resource_capacity_evaluation::overload_penalty,
};
use crate::{Constraint, CountValueTier, Error, Problem, Score, validate};

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
            Constraint::CountValue {
                variables,
                value,
                min,
                max,
                weight,
                tier,
            } => {
                let count = variables
                    .iter()
                    .filter(|&&index| values[index] == *value)
                    .count();
                let component = match tier {
                    CountValueTier::Hard => &mut result.hard,
                    CountValueTier::Soft => &mut result.soft,
                };
                let penalty =
                    i64::try_from(-(distance(count, *min, *max) as i128) * i128::from(*weight))
                        .map_err(|_| Error::ScoreOverflow)?;
                *component = component.checked_add(penalty).ok_or(Error::ScoreOverflow)?;
            }
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
            Constraint::Precedence {
                before,
                after,
                min_gap,
            } => {
                let penalty = shortfall(values[*before], values[*after], *min_gap);
                let hard = i128::from(result.hard) - penalty;
                result.hard = i64::try_from(hard).map_err(|_| Error::ScoreOverflow)?;
            }
            Constraint::NoOverlap { intervals } => {
                let penalty = super::no_overlap::overlap_score(intervals, |i| values[i])?;
                let penalty = i64::try_from(penalty).map_err(|_| Error::ScoreOverflow)?;
                result.hard = result
                    .hard
                    .checked_add(penalty)
                    .ok_or(Error::ScoreOverflow)?;
            }
            Constraint::ResourceCapacity { items, limits } => {
                let penalty = overload_penalty(items, limits, |index| values[index])?;
                let hard = i128::from(result.hard)
                    .checked_sub(penalty)
                    .ok_or(Error::ScoreOverflow)?;
                result.hard = i64::try_from(hard).map_err(|_| Error::ScoreOverflow)?;
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
