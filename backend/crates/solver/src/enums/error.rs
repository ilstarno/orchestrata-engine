#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyDomain(usize),
    InvalidVariable(usize),
    InvalidWeight,
    InvalidCountValue {
        constraint: usize,
        reason: super::count_value_error::CountValueError,
    },
    TooManyAssignments,
    ScoreOverflow,
    InvalidSolution,
}
