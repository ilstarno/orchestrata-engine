// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::OwnerSelectionReason;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerSelection<'a> {
    Assigned {
        representative_id: &'a str,
        workload: u64,
        eligible_count: usize,
        reason: OwnerSelectionReason,
    },
    NoEligibleRepresentative,
}
