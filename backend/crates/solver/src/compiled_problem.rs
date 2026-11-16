use crate::{Constraint, Error, Problem, validate};

/// A validated problem with an index of which constraints each variable affects.
pub struct CompiledProblem<'problem> {
    pub(super) problem: &'problem Problem,
    incident: Vec<Vec<usize>>,
}

impl<'problem> CompiledProblem<'problem> {
    pub fn new(problem: &'problem Problem) -> Result<Self, Error> {
        validate(problem)?;

        let mut incident = vec![Vec::new(); problem.variables.len()];
        for (constraint_index, constraint) in problem.constraints.iter().enumerate() {
            let mut record = |variable: usize| {
                let entries = &mut incident[variable];
                if entries.last() != Some(&constraint_index) {
                    entries.push(constraint_index);
                }
            };

            match constraint {
                Constraint::AllDifferent { variables, .. } => {
                    variables.iter().copied().for_each(record);
                }
                Constraint::NotEqualOffset { left, right, .. } => {
                    record(*left);
                    record(*right);
                }
                Constraint::PreferValue { variable, .. } => record(*variable),
            }
        }

        Ok(Self { problem, incident })
    }

    /// Lists each incident constraint once, in source order.
    pub fn incident_constraints(&self, variable: usize) -> Option<&[usize]> {
        self.incident.get(variable).map(Vec::as_slice)
    }
}
