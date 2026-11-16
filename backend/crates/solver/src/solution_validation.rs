use std::collections::{BTreeMap, HashSet};

use crate::{
    Constraint, CountValueTier, Problem, ResourceLimit, ResourceLoad, Score, Solution,
    SolutionValidationError, validate,
};

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
            Constraint::CountValue {
                variables,
                value,
                min,
                max,
                weight,
                tier,
            } => {
                let mut count = 0;
                for &variable in variables {
                    if values[variable] == *value {
                        count += 1;
                    }
                }
                if !(*min..=*max).contains(&count) {
                    if *tier == CountValueTier::Hard {
                        return Err(SolutionValidationError::HardConstraintViolation(index));
                    }
                    let difference = if count < *min {
                        *min - count
                    } else {
                        count - *max
                    };
                    let penalty = (difference as i128) * i128::from(*weight);
                    soft_penalty = soft_penalty
                        .saturating_add(penalty)
                        .min(-i128::from(i64::MIN) + 1);
                }
            }
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
            Constraint::Precedence {
                before,
                after,
                min_gap,
            } => {
                let actual_gap = i128::from(values[*after]) - i128::from(values[*before]);
                if actual_gap < i128::from(*min_gap) {
                    return Err(SolutionValidationError::HardConstraintViolation(index));
                }
            }
            Constraint::NoOverlap { intervals } => {
                // Verify separately from the solver's start-ordered overlap scan
                // and the scorer's sweep. Empty intervals occupy no time.
                let mut ordered_by_end: Vec<_> = intervals
                    .iter()
                    .filter(|interval| interval.duration > 0)
                    .map(|interval| {
                        let start = i128::from(values[interval.start]);
                        (start + i128::from(interval.duration), start)
                    })
                    .collect();
                ordered_by_end.sort_unstable();
                if ordered_by_end.windows(2).any(|pair| pair[0].0 > pair[1].1) {
                    return Err(SolutionValidationError::HardConstraintViolation(index));
                }
            }
            Constraint::ResourceCapacity { items, limits } => {
                if !verify_capacity(items, limits, values) {
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

fn verify_capacity(items: &[ResourceLoad], limits: &[ResourceLimit], values: &[i64]) -> bool {
    let mut remaining: BTreeMap<_, _> = limits
        .iter()
        .map(|limit| (limit.resource, limit.capacity))
        .collect();
    for item in items {
        let Some(available) = remaining.get_mut(&values[item.variable]) else {
            return false;
        };
        let Some(new_remaining) = available.checked_sub(item.load) else {
            return false;
        };
        *available = new_remaining;
    }
    true
}
