#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Objective {
    MaxAssigned,
    MinTotalCost {
        service_field: String,
        penalty_field: String,
        unit: String,
    },
    MinEdgeSum(String),
}
