use crate::SolutionValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolutionComparisonError {
    VariableCountMismatch,
    /// The previous model has a blank or repeated variable identity at this index.
    InvalidVariableIdentity(usize),
    /// The variable names differ at this index.
    VariableIdentityMismatch(usize),
    InvalidPreviousSolution(SolutionValidationError),
    InvalidUpdatedSolution(SolutionValidationError),
}
