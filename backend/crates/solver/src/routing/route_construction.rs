use super::{CourierRoute, RouteConstructionStop};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteConstruction {
    /// Nonempty routes in original courier order, with original shipment indices.
    pub routes: Vec<CourierRoute>,
    /// Original shipment indices without committed routes, in input order.
    /// Unassigned shipments are not proof of infeasibility.
    pub unassigned: Vec<usize>,
    /// Number of candidate route simulations, excluding independent verification.
    pub evaluated: u32,
    pub stop: RouteConstructionStop,
}
