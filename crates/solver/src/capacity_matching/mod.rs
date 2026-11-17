mod assignment;
mod error;
mod problem;
mod solve;

pub use assignment::CapacityAssignment;
pub use error::CapacityMatchError;
pub use problem::{CAPACITY_MATCH_MAX_RECORDS, CapacityMatchProblem};
pub use solve::match_capacity;

#[cfg(test)]
#[path = "../../../../backend_tests/rust/capacity_matching/matching.rs"]
mod tests;
