// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::BackendShare;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendAllocation<'a> {
    pub demand: u64,
    pub served: u64,
    pub unserved: u64,
    /// u128 preserves aggregate headroom over multiple full-sized u64 backends.
    pub available: u128,
    /// One outcome per supplied backend, sorted by ascending backend ID.
    pub backend_shares: Vec<BackendShare<'a>>,
}
