use super::{Courier, Shipment};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingProblem {
    pub couriers: Vec<Courier>,
    pub shipments: Vec<Shipment>,
}
