use crate::{SearchStop, Solution};

/// A bounded search result that makes no claim of optimality or infeasibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchOutcome {
    FeasibleIncumbent {
        solution: Solution,
        stop: SearchStop,
    },
    NoFeasibleIncumbent {
        evaluated: u64,
        stop: SearchStop,
    },
}
