mod assignment;
mod choice;
mod controlled_match_error;
mod edge;
mod network;
pub mod penalized;
mod problem;
mod solve;

pub use assignment::WeightedMatchAssignment;
pub use choice::WorkerCost;
pub use controlled_match_error::ControlledMatchError;
pub use problem::WeightedMatchProblem;
pub use solve::{match_capacity_min_cost, match_capacity_min_cost_controlled};

#[cfg(test)]
#[path = "../../../../../backend_tests/rust/capacity_matching/weighted.rs"]
mod tests;
