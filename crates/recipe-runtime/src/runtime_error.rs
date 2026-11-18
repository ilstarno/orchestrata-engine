use crate::assignment::AssignmentError;
use orchestrata_solver::capacity_matching::CapacityMatchError;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum RuntimeError {
    Input(AssignmentError),
    NamedSkillRecipeRequired,
    Solver(CapacityMatchError),
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(e) => e.fmt(f),
            Self::NamedSkillRecipeRequired => {
                f.write_str("Use a version 2 named skill assignment recipe.")
            }
            Self::Solver(e) => e.fmt(f),
        }
    }
}
impl Error for RuntimeError {}
impl From<AssignmentError> for RuntimeError {
    fn from(error: AssignmentError) -> Self {
        Self::Input(error)
    }
}
