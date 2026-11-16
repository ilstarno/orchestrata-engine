use super::{
    count_value_distance::distance, precedence_shortfall::shortfall,
    resource_capacity_evaluation::exceeds_capacity,
};
use crate::{Constraint, CountValueTier, Problem};

pub(super) fn candidate_penalty(problem: &Problem, values: &[i64]) -> Option<i128> {
    if problem
        .constraints
        .iter()
        .any(|rule| violates_hard_rule(rule, values))
    {
        return None;
    }

    Some(
        problem
            .constraints
            .iter()
            .filter_map(|rule| match rule {
                Constraint::CountValue {
                    variables,
                    value,
                    min,
                    max,
                    weight,
                    tier: CountValueTier::Soft,
                } => {
                    let count = variables
                        .iter()
                        .filter(|&&index| values[index] == *value)
                        .count();
                    Some((distance(count, *min, *max) as i128) * i128::from(*weight))
                }
                Constraint::PreferValue {
                    variable,
                    value,
                    soft_weight,
                } if values[*variable] != *value => Some(i128::from(*soft_weight)),
                _ => None,
            })
            // Penalties beyond this sentinel cannot be returned in a Score.
            .fold(0_i128, |total, penalty| {
                total.saturating_add(penalty).min(-i128::from(i64::MIN) + 1)
            }),
    )
}

fn violates_hard_rule(constraint: &Constraint, values: &[i64]) -> bool {
    match constraint {
        Constraint::CountValue {
            variables,
            value,
            min,
            max,
            tier,
            ..
        } => {
            if *tier == CountValueTier::Soft {
                return false;
            }
            let count = variables
                .iter()
                .filter(|&&index| values[index] == *value)
                .count();
            distance(count, *min, *max) != 0
        }
        Constraint::AllDifferent { variables, .. } => {
            variables.iter().enumerate().any(|(i, &left)| {
                variables[i + 1..]
                    .iter()
                    .any(|&right| values[left] == values[right])
            })
        }
        Constraint::NotEqualOffset {
            left,
            right,
            offset,
            ..
        } => i128::from(values[*left]) - i128::from(values[*right]) == i128::from(*offset),
        Constraint::Precedence {
            before,
            after,
            min_gap,
        } => shortfall(values[*before], values[*after], *min_gap) != 0,
        Constraint::NoOverlap { intervals } => {
            super::no_overlap::has_overlap(intervals, |i| values[i])
        }
        Constraint::ResourceCapacity { items, limits } => {
            exceeds_capacity(items, limits, |index| values[index])
        }
        Constraint::PreferValue { .. } => false,
    }
}
