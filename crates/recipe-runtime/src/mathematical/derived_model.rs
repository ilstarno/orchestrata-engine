use super::{CompiledModel, DerivedModelSpec, ModelError, ModelSolution};
pub fn parse_derived_model(xml: &str) -> Result<DerivedModelSpec, ModelError> {
    super::parse::parse_version(xml, "3").map(DerivedModelSpec)
}
pub fn compile_derived_model(
    spec: &DerivedModelSpec,
    json: &str,
) -> Result<CompiledModel, ModelError> {
    super::compile_model(&spec.0, json)
}
pub fn run_derived_model(xml: &str, json: &str) -> Result<ModelSolution, ModelError> {
    super::solve_model(compile_derived_model(&parse_derived_model(xml)?, json)?)
}
