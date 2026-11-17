// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::StalenessPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsRequest<'a> {
    /// Highest preference first. Any region absent from this list is forbidden.
    pub approved_regions: &'a [&'a str],
    pub staleness: StalenessPolicy,
}
