// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Availability snapshot for one candidate with an opaque stable identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Representative<'a> {
    pub id: &'a str,
    pub territory: &'a str,
    pub language: &'a str,
    pub available: bool,
    pub workload: u64,
}
