use std::mem;

use crate::module::reference_solver;
use crate::{Error, Problem, Solution, validate};

use super::domain::Domain;

/// An owned planning problem verified before search and normalized without adding options.
#[derive(Debug)]
pub struct PreparedProblem(Problem);

impl PreparedProblem {
    pub fn prepare(mut problem: Problem) -> Result<Self, Error> {
        validate(&problem)?;
        for variable in &mut problem.variables {
            variable.domain = Domain::from_options(mem::take(&mut variable.domain)).into_options();
        }
        Ok(Self(problem))
    }

    /// Solve all distinct assignments, preserving declared order for ties.
    pub fn solve(&self, max_evaluations: u64) -> Result<Option<Solution>, Error> {
        reference_solver::solve_validated(&self.0, max_evaluations)
    }
}
