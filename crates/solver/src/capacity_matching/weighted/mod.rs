mod assignment;
mod choice;
mod edge;
mod network;
mod problem;
mod solve;

pub use assignment::WeightedMatchAssignment;
pub use choice::WorkerCost;
pub use problem::WeightedMatchProblem;
pub use solve::match_capacity_min_cost;

#[cfg(test)]
#[path = "../../../../../backend_tests/rust/capacity_matching/weighted.rs"]
mod tests;
