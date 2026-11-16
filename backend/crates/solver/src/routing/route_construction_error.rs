use super::RouteVerificationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteConstructionError {
    TooManyCouriers(usize),
    TooManyShipments(usize),
    InsertionLimitTooHigh(u32),
    InvalidProblem(RouteVerificationError),
    VerificationFailed(RouteVerificationError),
}
