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

mod certificate;
mod verification_error;
mod verify;
pub use certificate::WeightedMatchCertificate;
pub use verification_error::WeightedVerificationError;
pub use verify::verify_weighted_assignment;

mod feasibility;
pub use feasibility::verify_weighted_feasibility;
mod weighted_objective;
pub use weighted_objective::WeightedObjective;
mod weighted_termination;
pub use weighted_termination::WeightedTermination;
mod certificate_evaluation;
pub use certificate_evaluation::CertificateEvaluation;
mod weighted_evidence;
pub use weighted_evidence::WeightedEvidence;
mod weighted_report;
pub use weighted_report::WeightedReport;
mod solve_report;
pub use solve_report::{assess_weighted_assignment, solve_weighted_report};
