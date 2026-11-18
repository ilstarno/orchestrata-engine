use super::CompiledModel;
#[derive(Debug)]
pub struct CompiledPenaltyModel {
    pub(super) model: CompiledModel,
    pub(super) penalties: Vec<u32>,
    pub(super) unit: String,
}
