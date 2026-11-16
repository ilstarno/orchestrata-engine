use crate::{Constraint, Problem};

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
                Constraint::PreferValue {
                    variable,
                    value,
                    soft_weight,
                } if values[*variable] != *value => Some(i128::from(*soft_weight)),
                _ => None,
            })
            .sum(),
    )
}

fn violates_hard_rule(constraint: &Constraint, values: &[i64]) -> bool {
    match constraint {
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
        Constraint::PreferValue { .. } => false,
    }
}
