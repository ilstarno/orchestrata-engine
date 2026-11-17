// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::DnsWeight;

/// Advisory relative weights for new lookups, not a commitment about request
/// counts, propagation, or currently active traffic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DnsPolicy<'a> {
    /// None means there was no usable approved region; all weights are zero.
    pub selected_region: Option<&'a str>,
    /// One outcome per supplied endpoint, sorted by ascending endpoint ID.
    pub endpoint_weights: Vec<DnsWeight<'a>>,
}
