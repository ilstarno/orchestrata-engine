pub struct CompiledRoute {
    pub(super) problem: orchestrata_solver::routing::RoutingProblem,
    pub(super) travel: super::compile::Travel,
    pub(super) snapshot: super::snapshot::Snapshot,
    pub(super) spec: super::RouteSpec,
    pub(super) budget: u32,
}
