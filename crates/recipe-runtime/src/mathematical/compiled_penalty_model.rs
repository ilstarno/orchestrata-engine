use super::bound_assignment::BoundAssignment;
#[derive(Debug)]
pub struct CompiledPenaltyModel {
    pub(super) model: BoundAssignment,
    pub(super) penalties: Vec<u32>,
    pub(super) unit: String,
}
