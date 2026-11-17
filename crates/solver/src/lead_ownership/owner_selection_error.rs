// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerSelectionError {
    BlankRequirements,
    InvalidRepresentativeId,
    DuplicateRepresentativeId,
}
