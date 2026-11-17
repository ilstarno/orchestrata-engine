// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::StalenessPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationRequest<'a> {
    /// Unlisted pools cannot contribute capacity to this request.
    pub pool: &'a str,
    /// Additional demand in the same unit and interval as backend capacity.
    pub demand: u64,
    pub staleness: StalenessPolicy,
}
