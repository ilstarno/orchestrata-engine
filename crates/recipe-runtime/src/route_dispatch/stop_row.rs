use serde::Deserialize;

use super::window::Window;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopRow {
    pub location: String,
    pub window: Window,
    pub service_minutes: u64,
}
