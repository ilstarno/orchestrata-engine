use orchestrata_solver::routing::TimeWindow;
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub start: u64,
    pub end: u64,
}

impl From<Window> for TimeWindow {
    fn from(value: Window) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}
