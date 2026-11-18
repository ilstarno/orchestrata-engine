/// Disjoint within each courier; every pair is incompatible even with optimistic travel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourierClique {
    pub courier: usize,
    pub shipments: Vec<usize>,
}
