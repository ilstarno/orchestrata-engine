use serde::Deserialize;

use super::{courier_row::CourierRow, leg_row::LegRow, shipment_row::ShipmentRow};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub locations: Vec<String>,
    pub couriers: Vec<CourierRow>,
    pub shipments: Vec<ShipmentRow>,
    pub travel_legs: Vec<LegRow>,
}
