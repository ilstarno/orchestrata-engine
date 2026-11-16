// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{CapacityEnvelope, HealthSnapshot};

/// Capacity and health reported by the caller for a backend in one pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backend<'a> {
    pub id: &'a str,
    pub pool: &'a str,
    pub admitted: bool,
    pub observation: HealthSnapshot,
    pub capacity: CapacityEnvelope,
}
