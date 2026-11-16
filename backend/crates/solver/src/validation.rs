use crate::{Constraint, Error, Problem};

pub fn validate(problem: &Problem) -> Result<(), Error> {
    for (index, variable) in problem.variables.iter().enumerate() {
        if variable.domain.is_empty() {
            return Err(Error::EmptyDomain(index));
        }
    }

    for constraint in &problem.constraints {
        let (weight, indices): (i64, &[usize]) = match constraint {
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
