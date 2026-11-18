mod assignment;
mod job;
mod problem;
mod solve;

pub use assignment::PenaltyMatchAssignment;
pub use job::PenaltyJob;
pub use problem::PenaltyMatchProblem;
pub use solve::match_capacity_with_penalties;

#[cfg(test)]
#[path = "../../../../../../backend_tests/rust/capacity_matching/penalized.rs"]
mod tests;
