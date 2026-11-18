use serde::Deserialize;

use super::mode::Mode;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegRow {
    pub mode: Mode,
    pub from: String,
    pub to: String,
    pub minutes: u64,
}
