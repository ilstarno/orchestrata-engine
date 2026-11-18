use serde::Deserialize;

use super::{mode::Mode, stop_row::StopRow};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShipmentRow {
    pub id: String,
    pub pickup: StopRow,
    pub delivery: StopRow,
    pub weight_kg: u64,
    pub allowed_modes: Vec<Mode>,
}
