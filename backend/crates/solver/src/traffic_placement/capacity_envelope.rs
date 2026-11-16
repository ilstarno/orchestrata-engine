// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Capacity in one consistent unit. DNS uses relative capacity points;
/// backend planning uses demand and capacity over one common interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapacityEnvelope {
    pub configured_limit: u64,
    pub committed: u64,
    pub reserve: u64,
}

impl CapacityEnvelope {
    /// Return zero if committed load and reserve exhaust the configured limit.
    pub const fn headroom(self) -> u64 {
        self.configured_limit
            .saturating_sub(self.committed)
            .saturating_sub(self.reserve)
    }
}
