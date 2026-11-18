use super::unsigned_field::UnsignedField;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Objective {
    MaxAssigned,
    MinTotalCost {
        service_field: UnsignedField,
        penalty_field: UnsignedField,
        unit: String,
    },
    MinEdgeSum(UnsignedField),
}
