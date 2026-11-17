use std::mem;

use super::{constraint_contribution::contribution, wide_score::WideScore};
use crate::{CompiledProblem, Error, MoveTrial, Score};

/// Maintains a candidate assignment and recalculates only affected constraints.
pub struct IncrementalScorer<'compiled> {
    compiled: &'compiled CompiledProblem<'compiled>,
    pub(super) values: Vec<i64>,
    pub(super) scores: Vec<WideScore>,
    pub(super) total: WideScore,
}

impl<'compiled> IncrementalScorer<'compiled> {
    pub fn new(
        compiled: &'compiled CompiledProblem<'compiled>,
        values: &[i64],
    ) -> Result<Self, Error> {
        let problem = compiled.problem;
        if values.len() != problem.variables.len() {
            return Err(Error::InvalidVariable(values.len()));
        }
        for (index, (variable, value)) in problem.variables.iter().zip(values).enumerate() {
            if !variable.domain.contains(value) {
                return Err(Error::InvalidVariable(index));
            }
        }

        let mut scores = Vec::with_capacity(problem.constraints.len());
        let mut total = WideScore::default();
        for constraint in &problem.constraints {
            let score = contribution(constraint, |index| values[index])?;
            total = total.add(score)?;
            scores.push(score);
        }

        Ok(Self {
            compiled,
            values: values.to_vec(),
            scores,
            total,
        })
    }

    pub fn values(&self) -> &[i64] {
        &self.values
    }

    /// Returns the same diagnostic score as full scoring, including overflow behavior.
    pub fn score(&self) -> Result<Score, Error> {
        self.total.narrow()
    }

    /// Checks the hard score without narrowing either score component.
    pub fn is_feasible(&self) -> bool {
        self.total.hard == 0
    }

    /// Tries a one-variable replacement. Dropping the trial rolls it back unless committed.
    pub fn try_move(
        &mut self,
        variable: usize,
        value: i64,
    ) -> Result<MoveTrial<'_, 'compiled>, Error> {
        let problem = self.compiled.problem;
        let domain = &problem
            .variables
            .get(variable)
            .ok_or(Error::InvalidVariable(variable))?
            .domain;
        if !domain.contains(&value) {
            return Err(Error::InvalidVariable(variable));
        }

        let previous_value = self.values[variable];
        let previous_total = self.total;
        let mut replacements = Vec::new();
        if previous_value != value {
            let affected = self
                .compiled
                .incident_constraints(variable)
                .unwrap_or_default();
            replacements.reserve(affected.len());

            // Remove all affected contributions before adding their replacements.
            // This avoids intermediate overflow when the final score fits.
            let mut total = affected.iter().fold(previous_total, |total, &index| {
                total.remove(self.scores[index])
            });

            for &index in affected {
                let score = contribution(&problem.constraints[index], |position| {
                    if position == variable {
                        value
                    } else {
                        self.values[position]
                    }
                })?;
                total = total.add(score)?;
                replacements.push((index, score));
            }

            for (index, score) in &mut replacements {
                mem::swap(&mut self.scores[*index], score);
            }
            self.total = total;
            self.values[variable] = value;
        }

        Ok(MoveTrial::new(
            self,
            variable,
            previous_value,
            previous_total,
            replacements,
        ))
    }
}
