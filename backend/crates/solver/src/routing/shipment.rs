use super::{CourierMode, Stop};

/// One load that must be picked up and delivered by the same courier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shipment {
    pub pickup: Stop,
    pub delivery: Stop,
    pub weight: u64,
    pub allowed_modes: Vec<CourierMode>,
}
