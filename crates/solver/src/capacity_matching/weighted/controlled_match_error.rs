use crate::capacity_matching::CapacityMatchError;

/// A rejected problem or an explicit interruption requested by the caller.
#[derive(Debug, PartialEq, Eq)]
pub enum ControlledMatchError<E> {
    InvalidProblem(CapacityMatchError),
    Cancelled(E),
}

impl<E> std::fmt::Display for ControlledMatchError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled(_) => f.write_str("Solve cancelled at a cooperative checkpoint."),
            Self::InvalidProblem(error) => error.fmt(f),
        }
    }
}
impl<E: std::fmt::Debug> std::error::Error for ControlledMatchError<E> {}
