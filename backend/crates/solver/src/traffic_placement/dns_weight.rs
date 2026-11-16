// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::DnsDisposition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsWeight<'a> {
    pub endpoint_id: &'a str,
    pub weight: u16,
    pub disposition: DnsDisposition,
}
