// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// A traversable directed edge; use consistent travel-time units throughout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Road {
    pub from: usize,
    pub to: usize,
    pub travel_time: i64,
}
