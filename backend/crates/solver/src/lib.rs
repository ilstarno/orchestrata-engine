#[path = "module.rs"]
mod module;
pub mod routing;

pub use module::assignment_change::AssignmentChange;
pub use module::assignment_change_reason::AssignmentChangeReason;
pub use module::compare_solutions::compare_solutions;
pub use module::compiled_problem::CompiledProblem;
pub use module::enums::{constraint::Constraint, error::Error};
pub use module::incremental_scorer::IncrementalScorer;
pub use module::local_search::search_locally;
pub use module::move_trial::MoveTrial;
pub use module::problem::Problem;
pub use module::reference_solver::solve;
pub use module::score::Score;
pub use module::scoring::score;
pub use module::search_outcome::SearchOutcome;
pub use module::search_stop::SearchStop;
pub use module::solution::Solution;
pub use module::solution_comparison::SolutionComparison;
pub use module::solution_comparison_error::SolutionComparisonError;
pub use module::solution_validation::verify_solution;
pub use module::solution_validation_error::SolutionValidationError;
pub use module::validation::validate;
pub use module::variable::Variable;
