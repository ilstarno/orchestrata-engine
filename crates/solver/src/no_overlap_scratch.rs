// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::no_overlap::ranges;
use crate::ScheduledInterval;

/// Per-solve storage for exact feasibility checks, never shared by solvers.
#[derive(Default)]
pub(super) struct NoOverlapScratch {
    starts: Vec<(i128, i128)>,
}

impl NoOverlapScratch {
    pub(super) fn has_overlap(
        &mut self,
        intervals: &[ScheduledInterval],
        value_at: impl Fn(usize) -> i64,
    ) -> bool {
        let mut ranges = ranges(intervals, value_at);
        let (Some(left), Some(right)) = (ranges.next(), ranges.next()) else {
            return false;
        };

        // Two occupied half-open intervals need no sorting or allocation.
        let Some(third) = ranges.next() else {
            return left.0 < right.1 && right.0 < left.1;
        };

        self.starts.clear();
        self.starts.extend([left, right, third]);
        self.starts.extend(ranges);
        self.starts.sort_unstable();
        self.starts.windows(2).any(|pair| pair[0].1 > pair[1].0)
    }
}
