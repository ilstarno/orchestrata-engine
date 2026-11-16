// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Normalized, exact routing codes; no customer record is required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerRequirements<'a> {
    pub territory: &'a str,
    pub language: &'a str,
}
