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
    InvalidPrecedence {
        constraint: usize,
        reason: super::precedence_error::PrecedenceError,
    },
    InvalidNoOverlap {
        constraint: usize,
        reason: super::no_overlap_error::NoOverlapError,
    },
    InvalidResourceCapacity(ResourceCapacityError),
    TooManyAssignments,
    ScoreOverflow,
    InvalidSolution,
}
