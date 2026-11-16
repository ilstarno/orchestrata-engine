// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// A half-open UTC minute range. The caller resolves time zones before solving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

impl TimeRange {
    pub(super) const fn contains(self, other: Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    pub(super) const fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}
