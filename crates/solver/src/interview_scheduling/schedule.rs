// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::AppointmentResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterviewSchedule<'a> {
    /// Ordered by appointment request ID, independent of the input order.
    pub appointments: Vec<AppointmentResult<'a>>,
    /// Counts moves, panel changes, and holds of previously proposed meetings.
    pub changed_previous: usize,
    /// False means the search returned a feasible incumbent, not a proof.
    pub proven_optimal: bool,
    pub search_nodes: u32,
}
