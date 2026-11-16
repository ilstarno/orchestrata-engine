use super::{CourierMode, LocationId, TimeWindow};

/// A courier starts empty at its depot and must return there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Courier {
    pub mode: CourierMode,
    pub depot: LocationId,
    pub shift: TimeWindow,
    pub capacity: u64,
}
