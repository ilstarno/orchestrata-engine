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
