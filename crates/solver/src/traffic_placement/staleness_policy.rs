// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Compare supplied timestamps against a fixed evaluation time; no system clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StalenessPolicy {
    pub evaluated_at_seconds: u64,
    /// A check of exactly this age still counts as current.
    pub maximum_age_seconds: u64,
}
