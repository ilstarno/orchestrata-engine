use super::{
    constraint::Constraint, decision::Decision, input_binding::InputBinding, objective::Objective,
};
use std::collections::BTreeMap;
#[derive(Debug)]
pub struct ModelSpec {
    pub(super) inputs: BTreeMap<String, InputBinding>,
    pub(super) decision: Decision,
    pub(super) constraints: Vec<Constraint>,
    pub(super) objectives: Vec<Objective>,
}
