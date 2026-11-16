// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::TimeRange;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MeetingOption {
    pub(super) start: u64,
    pub(super) index_panel: Vec<usize>,
    pub(super) occupied: TimeRange,
    pub(super) retained: bool,
}
