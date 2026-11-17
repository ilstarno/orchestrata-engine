use std::collections::HashMap;

use super::{
    count_value_distance::distance, precedence_shortfall::shortfall,
    resource_capacity_evaluation::overload_penalty, wide_score::WideScore,
};
use crate::{Constraint, CountValueTier, Error};

pub(super) fn contribution(
    constraint: &Constraint,
    value_at: impl Fn(usize) -> i64,
) -> Result<WideScore, Error> {
    let mut result = WideScore::default();

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
                .filter(|&&index| value_at(index) == *value)
                .count();
            let penalty = -(distance(count, *min, *max) as i128) * i128::from(*weight);
            match tier {
                CountValueTier::Hard => result.hard = penalty,
                CountValueTier::Soft => result.soft = penalty,
            }
        }
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
        Constraint::Precedence {
            before,
            after,
            min_gap,
        } => {
            result.hard = -shortfall(value_at(*before), value_at(*after), *min_gap);
        }
        Constraint::NoOverlap { intervals } => {
            result.hard = super::no_overlap::overlap_score(intervals, value_at)?;
        }
        Constraint::ResourceCapacity { items, limits } => {
            result.hard = -overload_penalty(items, limits, value_at)?;
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
