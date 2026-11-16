use std::collections::HashSet;

use super::resource_capacity_validation::validate_resource_capacity;
use crate::{Constraint, CountValueError, Error, Problem};

pub fn validate(problem: &Problem) -> Result<(), Error> {
    for (index, variable) in problem.variables.iter().enumerate() {
        if variable.domain.is_empty() {
            return Err(Error::EmptyDomain(index));
        }
    }

    for (constraint_index, constraint) in problem.constraints.iter().enumerate() {
        let (weight, indices): (i64, &[usize]) = match constraint {
            Constraint::CountValue {
                variables,
                min,
                max,
                weight,
                ..
            } => {
                let invalid = |reason| Error::InvalidCountValue {
                    constraint: constraint_index,
                    reason,
                };
                if variables.is_empty() {
                    return Err(invalid(CountValueError::EmptyScope));
                }
                if min > max || *max > variables.len() {
                    return Err(invalid(CountValueError::InvalidBounds));
                }
                if *weight <= 0 {
                    return Err(invalid(CountValueError::InvalidWeight));
                }
                let mut seen = HashSet::with_capacity(variables.len());
                for &index in variables {
                    if index >= problem.variables.len() {
                        return Err(invalid(CountValueError::UnknownVariable(index)));
                    }
                    if !seen.insert(index) {
                        return Err(invalid(CountValueError::DuplicateVariable(index)));
                    }
                }
                continue;
            }
            Constraint::AllDifferent {
                variables,
                hard_weight,
            } => (*hard_weight, variables),
            Constraint::NotEqualOffset {
                left,
                right,
                hard_weight,
                ..
            } => {
                if *hard_weight <= 0 {
                    return Err(Error::InvalidWeight);
                }
                if *left >= problem.variables.len() {
                    return Err(Error::InvalidVariable(*left));
                }
                if *right >= problem.variables.len() {
                    return Err(Error::InvalidVariable(*right));
                }
                continue;
            }
            Constraint::ResourceCapacity { items, limits } => {
                validate_resource_capacity(problem, items, limits)?;
                continue;
            }
            Constraint::PreferValue {
                variable,
                soft_weight,
                ..
            } => {
                if *soft_weight <= 0 {
                    return Err(Error::InvalidWeight);
                }
                if *variable >= problem.variables.len() {
                    return Err(Error::InvalidVariable(*variable));
                }
                continue;
            }
        };

        if weight <= 0 {
            return Err(Error::InvalidWeight);
        }

        for &index in indices {
            if index >= problem.variables.len() {
                return Err(Error::InvalidVariable(index));
            }
        }
    }

    Ok(())
}
