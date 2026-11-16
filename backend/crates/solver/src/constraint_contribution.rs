use std::collections::HashMap;

use super::wide_score::WideScore;
use crate::{Constraint, Error};

pub(super) fn contribution(
    constraint: &Constraint,
    value_at: impl Fn(usize) -> i64,
) -> Result<WideScore, Error> {
    let mut result = WideScore::default();

    match constraint {
        Constraint::AllDifferent {
            variables,
            hard_weight,
        } => {
            let mut counts = HashMap::<i64, i128>::new();
            for &variable in variables {
                let count = counts.entry(value_at(variable)).or_default();
                let new_penalty = count
                    .checked_mul(i128::from(*hard_weight))
                    .ok_or(Error::ScoreOverflow)?;
                result.hard = result
                    .hard
                    .checked_sub(new_penalty)
                    .ok_or(Error::ScoreOverflow)?;
                *count += 1;
            }
        }
        Constraint::NotEqualOffset {
            left,
            right,
            offset,
            hard_weight,
        } => {
            if i128::from(value_at(*left)) == i128::from(value_at(*right)) + i128::from(*offset) {
                result.hard = -i128::from(*hard_weight);
            }
        }
        Constraint::PreferValue {
            variable,
            value,
            soft_weight,
        } => {
            if value_at(*variable) != *value {
                result.soft = -i128::from(*soft_weight);
            }
        }
    }

    Ok(result)
}
