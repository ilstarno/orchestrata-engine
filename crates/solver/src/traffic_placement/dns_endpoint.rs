// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{CapacityEnvelope, HealthSnapshot};

/// A steering target identified by an opaque key, not an address or DNS record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsEndpoint<'a> {
    pub id: &'a str,
    pub region: &'a str,
    pub admitted: bool,
    pub observation: HealthSnapshot,
    pub capacity: CapacityEnvelope,
}
