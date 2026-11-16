// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::Health;

/// One operator observation. Times use integer seconds on the caller's clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub status: Health,
    pub checked_at_seconds: Option<u64>,
}
