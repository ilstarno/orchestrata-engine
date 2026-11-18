use orchestrata_solver::routing::CourierMode;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Vehicle,
    Bicycle,
    Walking,
}
impl From<Mode> for CourierMode {
    fn from(m: Mode) -> Self {
        match m {
            Mode::Vehicle => Self::Vehicle,
            Mode::Bicycle => Self::Bicycle,
            Mode::Walking => Self::Walking,
        }
    }
}
