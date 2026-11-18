use std::{error::Error, fmt};
#[derive(Debug)]
pub enum DispatchError {
    InvalidModel,
    InvalidInput,
    Limit,
    NativeFailure,
}
impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidModel => "Unsupported route-dispatch model",
            Self::InvalidInput => "Invalid route records",
            Self::Limit => "Route resource limit",
            Self::NativeFailure => "Route construction rejected",
        })
    }
}
impl Error for DispatchError {}
