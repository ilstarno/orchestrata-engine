#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyDomain(usize),
    InvalidVariable(usize),
    InvalidWeight,
    TooManyAssignments,
    ScoreOverflow,
    InvalidSolution,
}
