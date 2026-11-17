/// A malformed precedence rule is not evidence of an infeasible schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecedenceError {
    UnknownBeforeVariable(usize),
    UnknownAfterVariable(usize),
    SameVariable(usize),
}
