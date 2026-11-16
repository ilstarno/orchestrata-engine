use super::{LocationId, TimeWindow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stop {
    pub location: LocationId,
    /// Service must start and finish within this interval.
    pub window: TimeWindow,
    pub service_time: u64,
}
