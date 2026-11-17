// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::BackendDisposition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendShare<'a> {
    pub backend_id: &'a str,
    /// Usable headroom in the requested pool, zero when out of scope or unusable.
    pub available: u64,
    pub proposed: u64,
    pub disposition: BackendDisposition,
}
