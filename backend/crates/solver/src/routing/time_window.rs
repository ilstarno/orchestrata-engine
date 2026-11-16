/// Inclusive bounds on an integer clock, in the same units as travel and service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: u64,
    pub end: u64,
}

impl TimeWindow {
    pub(super) fn is_valid(self) -> bool {
        self.start <= self.end
    }

    pub(super) fn contains(self, time: u64) -> bool {
        (self.start..=self.end).contains(&time)
    }
}
