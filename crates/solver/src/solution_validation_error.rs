use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolutionValidationError {
    InvalidProblem(Error),
    WrongValueCount,
    ValueOutsideDomain(usize),
    HardConstraintViolation(usize),
    ScoreOverflow,
    ScoreMismatch,
}
