use super::{bound_assignment::BoundAssignment, coverage_goal::CoverageGoal};
#[derive(Debug)]
pub struct CompiledModel {
    pub(super) assignment: BoundAssignment,
    pub(super) goal: CoverageGoal,
}
