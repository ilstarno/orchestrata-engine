use super::{ModelError, ModelSolution, compile_model, parse_model, solve_model};
pub fn run_model(xml: &str, inputs: &str) -> Result<ModelSolution, ModelError> {
    solve_model(compile_model(&parse_model(xml)?, inputs)?)
}
