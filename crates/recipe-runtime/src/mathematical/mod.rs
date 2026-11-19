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
mod run_model;
pub use run_model::run_model;

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

mod boolean_field;
mod bound_assignment;
mod coverage_goal;
mod field_binding;
mod id_field;
mod read_records;
mod record_id;
mod record_value;
#[cfg(test)]
mod typed_record_tests;
mod unsigned_field;
mod validated_record;

mod assignment_domain;
mod derived_domain;
mod derived_model;
mod derived_model_spec;
mod parse_derived_domain;
mod row_equality;
pub use derived_model::{compile_derived_model, parse_derived_model, run_derived_model};
pub use derived_model_spec::DerivedModelSpec;

mod controlled_model_error;
mod controlled_solve;
mod finish_assignment;
pub use controlled_model_error::ControlledModelError;
pub use controlled_solve::solve_model_controlled;
