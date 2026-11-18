mod bounded_rows;
mod compile;
mod compiled_model;
mod constraint;
mod decision;
mod field_type;
mod input_binding;
mod model_assignment;
mod model_error;
mod model_solution;
mod model_spec;
mod objective;
mod parse;
mod solve;
mod strict_object;
mod xml_helpers;
pub use compile::compile_model;
pub use compiled_model::CompiledModel;
pub use model_assignment::ModelAssignment;
pub use model_error::ModelError;
pub use model_solution::ModelSolution;
pub use model_spec::ModelSpec;
pub use parse::parse_model;
pub use solve::solve_model;
pub fn run_model(xml: &str, inputs: &str) -> Result<ModelSolution, ModelError> {
    solve_model(compile_model(&parse_model(xml)?, inputs)?)
}

mod compiled_penalty_model;
mod penalty;
mod penalty_model_solution;
mod penalty_model_spec;
pub use compiled_penalty_model::CompiledPenaltyModel;
pub use penalty::{
    compile_penalty_model, parse_penalty_model, run_penalty_model, solve_penalty_model,
};
pub use penalty_model_solution::PenaltyModelSolution;
pub use penalty_model_spec::PenaltyModelSpec;
