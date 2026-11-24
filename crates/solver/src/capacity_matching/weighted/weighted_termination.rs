/// Solver execution only; does not describe verification success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightedTermination {
    Completed,
    NotRun,
}
