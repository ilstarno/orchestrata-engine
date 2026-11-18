use serde::Deserialize;

use super::{mode::Mode, window::Window};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CourierRow {
    pub id: String,
    pub mode: Mode,
    pub depot: String,
    pub shift: Window,
    pub capacity_kg: u64,
}
