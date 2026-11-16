use crate::ResourceCapacityError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyDomain(usize),
    InvalidVariable(usize),
    InvalidWeight,
    InvalidCountValue {
        constraint: usize,
        reason: super::count_value_error::CountValueError,
    },
    InvalidResourceCapacity(ResourceCapacityError),
    TooManyAssignments,
    ScoreOverflow,
    InvalidSolution,
}
