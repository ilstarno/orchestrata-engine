// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::UnusableReason;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DnsDisposition {
    WeightProposed,
    BelowWeightResolution,
    RegionNotApproved,
    EarlierRegionAvailable,
    Unusable(UnusableReason),
}
