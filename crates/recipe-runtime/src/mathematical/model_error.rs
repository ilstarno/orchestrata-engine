use std::{error::Error, fmt};
#[derive(Debug, PartialEq, Eq)]
pub enum ModelError {
    InvalidXml,
    UnsupportedSemantics,
    InvalidInput,
    ResourceLimit,
    Infeasible,
    SolverFailure,
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidXml => "Invalid bounded mathematical XML.",
            Self::UnsupportedSemantics => "Unsupported or inconsistent model semantics.",
            Self::InvalidInput => "Inputs must match the declared typed named records.",
            Self::ResourceLimit => "Model or input exceeds the supported resource bounds.",
            Self::Infeasible => "No assignment satisfies all required constraints.",
            Self::SolverFailure => "The validated model could not be solved.",
        })
    }
}
impl Error for ModelError {}
