use super::ModelError;

/// Controlled solving never returns a partially optimized assignment.
#[derive(Debug, PartialEq, Eq)]
pub enum ControlledModelError<E> {
    Cancelled(E),
    UnsupportedGoal,
    Model(ModelError),
}

impl<E> std::fmt::Display for ControlledModelError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled(_) => f.write_str("Solve cancelled at a cooperative checkpoint."),
            Self::Model(error) => error.fmt(f),
            Self::UnsupportedGoal => {
                f.write_str("Controlled solving supports only coverage-then-edge-cost models.")
            }
        }
    }
}
impl<E: std::fmt::Debug> std::error::Error for ControlledModelError<E> {}
