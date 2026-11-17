// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use crate::Variable;

/// A mixed-radix cursor over validated, nonempty domains.
pub(super) struct ExactAssignments<'a> {
    variables: &'a [Variable],
    values: Vec<i64>,
    active: Vec<(usize, usize)>,
}

impl<'a> ExactAssignments<'a> {
    pub(super) fn new(variables: &'a [Variable]) -> Self {
        Self {
            variables,
            values: variables
                .iter()
                .map(|variable| variable.domain[0])
                .collect(),
            active: variables
                .iter()
                .enumerate()
                .filter(|(_, variable)| variable.domain.len() > 1)
                .map(|(index, _)| (index, 0))
                .collect(),
        }
    }

    pub(super) fn values(&self) -> &[i64] {
        &self.values
    }

    /// Advance in declared order, updating only the axes touched by the carry.
    /// Singleton axes never change; repeated values still occupy distinct positions.
    pub(super) fn advance(&mut self) -> bool {
        for (index, position) in self.active.iter_mut().rev() {
            let domain = &self.variables[*index].domain;
            *position += 1;
            if *position < domain.len() {
                self.values[*index] = domain[*position];
                return true;
            }
            *position = 0;
            self.values[*index] = domain[0];
        }
        false
    }
}
