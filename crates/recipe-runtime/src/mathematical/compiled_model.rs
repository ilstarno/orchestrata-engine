use orchestrata_solver::capacity_matching::weighted::WorkerCost;
#[derive(Debug)]
pub struct CompiledModel {
    pub(super) items: Vec<String>,
    pub(super) resources: Vec<String>,
    pub(super) capacities: Vec<usize>,
    pub(super) domains: Vec<Vec<WorkerCost>>,
    pub(super) minimize_cost: bool,
    pub(super) nullable: bool,
}
