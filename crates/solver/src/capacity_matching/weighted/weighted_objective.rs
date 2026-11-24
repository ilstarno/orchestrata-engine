/// Lexicographic objective; costs never trade away coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightedObjective {
    MaximizeAssignedThenMinimizeSuppliedCost,
}
