#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchCoverageError {
    InvalidResult,
    Native(orchestrata_solver::routing::coverage::CoverageError),
}
